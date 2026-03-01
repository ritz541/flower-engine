mod app;
mod models;
mod ui;
mod ws;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io, time::{Duration, Instant}};

use app::App;
use models::WsMessage;
use tokio::sync::mpsc;

const TICK_RATE: Duration = Duration::from_millis(150); // Spinner + cursor animation

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let (tx_in, mut rx_in) = mpsc::unbounded_channel::<WsMessage>();
    let (tx_out, rx_out) = mpsc::unbounded_channel::<String>();

    tokio::spawn(async move {
        ws::start_ws_client(tx_in, rx_out).await;
    });

    let res = run_app(&mut terminal, &mut app, &mut rx_in, tx_out).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    rx_in: &mut mpsc::UnboundedReceiver<WsMessage>,
    tx_out: mpsc::UnboundedSender<String>,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if app.should_quit {
            return Ok(());
        }

        // --- Process WS Stream ---
        while let Ok(msg) = rx_in.try_recv() {
            match msg.event.as_str() {
                "sync_state" => {
                    app.status = "Synced".to_string();
                    if let Some(w_id) = msg.payload.metadata.world_id {
                        app.world_id = w_id;
                    }
                    if let Some(c_id) = msg.payload.metadata.character_id {
                        app.character_id = c_id;
                    }
                    if let Some(worlds) = msg.payload.metadata.available_worlds {
                        app.available_worlds = worlds;
                    }
                    if let Some(chars) = msg.payload.metadata.available_characters {
                        app.available_characters = chars;
                    }
                    if let Some(models) = msg.payload.metadata.available_models {
                        app.available_models = models;
                    }
                    if let Some(rules) = msg.payload.metadata.available_rules {
                        app.available_rules = rules;
                    }
                    if let Some(active) = msg.payload.metadata.active_rules {
                        app.active_rules = active;
                    }
                }
                "system_update" => {
                    app.add_system_message(msg.payload.content);
                }
                "chat_chunk" => {
                    app.append_chunk(&msg.payload.content);
                    if let Some(tps) = msg.payload.metadata.tokens_per_second {
                        app.tps = tps;
                    }
                    if let Some(model) = msg.payload.metadata.model {
                        app.active_model = model;
                    }
                }
                "chat_end" => {
                    if let Some(total) = msg.payload.metadata.total_tokens {
                        app.total_tokens += total;
                    }
                    app.finish_stream();
                    app.status = "Idle".to_string();
                }
                "error" => {
                    app.add_system_message(format!("✗ {}", msg.payload.content));
                    app.is_typing = false;
                }
                _ => {} 
            }
        }

        // --- Tick & Input Event Timeout ---
        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
            
        if event::poll(timeout)? {
            match event::read()? {
                Event::Mouse(mouse) => {
                    match mouse.kind {
                        MouseEventKind::ScrollUp => {
                            app.scroll = app.scroll.saturating_sub(2);
                        }
                        MouseEventKind::ScrollDown => {
                            app.scroll = app.scroll.saturating_add(2);
                        }
                        _ => {}
                    }
                }
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Esc => {
                            if app.is_typing {
                                // Cancel the active stream
                                let _ = tx_out.send("/cancel".to_string());
                                app.is_typing = false;
                            } else if app.show_popup {
                                app.show_popup = false;
                                app.popup_mode = app::PopupMode::None;
                                app.input.clear();
                            } else {
                                app.should_quit = true;
                            }
                        }
                        KeyCode::Tab => {
                            app.apply_hint();
                        }
                        KeyCode::Enter => {
                            if app.show_popup {
                                // --- Popup Selection Confirmed ---
                                let filtered = app.get_filtered_items();
                                let prefix = match app.popup_mode {
                                    app::PopupMode::World     => "/world select",
                                    app::PopupMode::Character => "/character select",
                                    app::PopupMode::Model     => "/model",
                                    app::PopupMode::Rules     => "/rules add",
                                    _                         => "",
                                };
                                
                                if !filtered.is_empty() && app.selected_index < filtered.len() {
                                    let selection = &filtered[app.selected_index];
                                    let cmd = format!("{} {}", prefix, selection.id);
                                    let _ = tx_out.send(cmd);
                                }
                                
                                app.show_popup = false;
                                app.popup_mode = app::PopupMode::None;
                                app.popup_search_query.clear();
                                app.input.clear();
                                
                            } else if let Some(msg) = app.submit_message() {
                                let _ = tx_out.send(msg);
                            }
                        }
                        KeyCode::Char(c) => {
                            if app.show_popup {
                                app.popup_search_query.push(c);
                                app.set_popup_index(0); // Reset index when searching
                            } else {
                                app.handle_char(c);
                                
                                // --- Trigger Popup on exact string match ---
                                let trigger = if app.input == "/world " && !app.available_worlds.is_empty() {
                                    Some(app::PopupMode::World)
                                } else if app.input == "/character " && !app.available_characters.is_empty() {
                                    Some(app::PopupMode::Character)
                                } else if app.input == "/model " && !app.available_models.is_empty() {
                                    Some(app::PopupMode::Model)
                                } else if app.input == "/rules " && !app.available_rules.is_empty() {
                                    Some(app::PopupMode::Rules)
                                } else {
                                    None
                                };

                                if let Some(mode) = trigger {
                                    app.show_popup = true;
                                    app.popup_mode = mode;
                                    app.set_popup_index(0);
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            if app.show_popup {
                                app.popup_search_query.pop();
                                app.set_popup_index(0);
                            } else {
                                app.handle_backspace();
                            }
                        }
                        KeyCode::Up => {
                            if app.show_popup {
                                let new_idx = app.selected_index.saturating_sub(1);
                                app.set_popup_index(new_idx);
                            } else {
                                app.scroll = app.scroll.saturating_sub(1);
                            }
                        }
                        KeyCode::Down => {
                            if app.show_popup {
                                let max = app.get_filtered_items().len().saturating_sub(1);
                                if app.selected_index < max {
                                    let new_idx = app.selected_index + 1;
                                    app.set_popup_index(new_idx);
                                }
                            } else {
                                app.scroll = app.scroll.saturating_add(1);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        
        // --- TICK UPDATE (cursor blink + spinner) ---
        if last_tick.elapsed() >= TICK_RATE {
            // app.cursor_state = !app.cursor_state; // BLINK DISABLED
            if app.is_typing {
                app.spinner_frame = (app.spinner_frame + 1) % 10;
            }
            last_tick = Instant::now();
        }
    }
}
