mod app;
mod models;
mod ui;
mod ws;

use crossterm::{
    event::{Event, EventStream, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io, time::Duration};

use app::App;
use models::WsMessage;
use tokio::sync::mpsc;
use futures_util::{StreamExt, FutureExt};

const TICK_RATE: Duration = Duration::from_millis(150); // Spinner + cursor animation

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let (tx_in, rx_in) = mpsc::unbounded_channel::<WsMessage>();
    let (tx_out, rx_out) = mpsc::unbounded_channel::<String>();

    tokio::spawn(async move {
        ws::start_ws_client(tx_in, rx_out).await;
    });

    let res = run_app(&mut terminal, &mut app, rx_in, tx_out).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
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
    mut rx_in: mpsc::UnboundedReceiver<WsMessage>,
    tx_out: mpsc::UnboundedSender<String>,
) -> io::Result<()> {
    let mut reader = EventStream::new();
    let mut last_tick = std::time::Instant::now();
    
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if app.should_quit {
            return Ok(());
        }

        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        tokio::select! {
            // --- Process WS Stream ---
            Some(msg) = rx_in.recv() => {
                match msg.event.as_str() {
                    "sync_state" => {
                        app.status = "Synced".to_string();
                        if let Some(model) = msg.payload.metadata.model { 
                            app.active_model = model.clone();
                            if let Some(confirmed) = msg.payload.metadata.model_confirmed {
                                app.model_confirmed = confirmed;
                            }
                            if let Some(info) = app.available_models.iter().find(|m| m.id == model) {
                                app.active_prompt_price = info.prompt_price;
                                app.active_completion_price = info.completion_price;
                            }
                        }
                        if let Some(w_id) = msg.payload.metadata.world_id { app.world_id = w_id; }
                        if let Some(c_id) = msg.payload.metadata.character_id { app.character_id = c_id; }
                        if let Some(worlds) = msg.payload.metadata.available_worlds { app.available_worlds = worlds; }
                        if let Some(chars) = msg.payload.metadata.available_characters { app.available_characters = chars; }
                        if let Some(models) = msg.payload.metadata.available_models { app.available_models = models; }
                        if let Some(rules) = msg.payload.metadata.available_rules { app.available_rules = rules; }
                        if let Some(active) = msg.payload.metadata.active_rules { app.active_rules = active; }
                        if let Some(sess_id) = msg.payload.metadata.session_id {
                            app.session_id = sess_id;
                        }
                        if let Some(sessions) = msg.payload.metadata.available_sessions {
                            app.available_sessions = sessions;
                        }
                    }
                    "chat_history" => {
                        if let Some(history) = msg.payload.metadata.history {
                            let converted = history.into_iter()
                                .map(|m| (m.role, m.content))
                                .collect();
                            app.load_history(converted);
                        }
                    }
                    "system_update" => {
                        app.add_system_message(msg.payload.content);
                        if let Some(model) = msg.payload.metadata.model { 
                            app.active_model = model.clone();
                            app.model_confirmed = true;
                            if let Some(info) = app.available_models.iter().find(|m| m.id == model) {
                                app.active_prompt_price = info.prompt_price;
                                app.active_completion_price = info.completion_price;
                            }
                        }
                        if let Some(sess_id) = msg.payload.metadata.session_id { app.session_id = sess_id; }
                    }
                    "chat_chunk" => {
                        app.append_chunk(&msg.payload.content);
                        if let Some(tps) = msg.payload.metadata.tokens_per_second { app.tps = tps; }
                        if let Some(model) = msg.payload.metadata.model { 
                            app.active_model = model.clone();
                            // Update prices for current model
                            if let Some(info) = app.available_models.iter().find(|m| m.id == model) {
                                app.active_prompt_price = info.prompt_price;
                                app.active_completion_price = info.completion_price;
                            }
                        }
                    }
                    "chat_end" => {
                        if let Some(total) = msg.payload.metadata.total_tokens { app.total_tokens += total; }
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
            // --- Process Terminal Events ---
            Some(Ok(event)) = reader.next().fuse() => {
                match event {
                    Event::Key(key) => {
                        match key.code {
                            KeyCode::Esc => {
                                if app.is_typing {
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
                            KeyCode::Tab => { app.apply_hint(); }
                            KeyCode::Enter => {
                                if app.show_popup {
                                    let filtered = app.get_filtered_items();
                                    if app.popup_mode == app::PopupMode::Commands {
                                        if !filtered.is_empty() && app.selected_index < filtered.len() {
                                            let selection = &filtered[app.selected_index];
                                            let cmd = selection.id.clone();

                                            // Handle "complete" commands immediately
                                            if !cmd.ends_with(' ') {
                                                if let Some(final_cmd) = app.submit_command_direct(cmd) {
                                                    let _ = tx_out.send(final_cmd);
                                                }
                                                app.show_popup = false;
                                                app.popup_mode = app::PopupMode::None;
                                                app.input.clear();
                                            } else {
                                                // Prefix commands: populate input and chain the next popup
                                                app.input = cmd.clone();
                                                app.popup_search_query.clear();

                                                let next_mode = if cmd.contains("/world select") { Some(app::PopupMode::World) }
                                                else if cmd.contains("/character select") { Some(app::PopupMode::Character) }
                                                else if cmd.contains("/model") { Some(app::PopupMode::Model) }
                                                else if cmd.contains("/rules add") { Some(app::PopupMode::Rules) }
                                                else if cmd.contains("/session continue") || cmd.contains("/session delete") { Some(app::PopupMode::Session) }
                                                else { None };

                                                if let Some(mode) = next_mode {
                                                    app.popup_mode = mode;
                                                    app.set_popup_index(0);
                                                } else {
                                                    app.show_popup = false;
                                                    app.popup_mode = app::PopupMode::None;
                                                }
                                            }
                                        }
                                        app.popup_search_query.clear();
                                    } else {
                                        let prefix = match app.popup_mode {
                                            app::PopupMode::World     => "/world select",
                                            app::PopupMode::Character => "/character select",
                                            app::PopupMode::Model     => "/model",
                                            app::PopupMode::Rules     => "/rules add",
                                            app::PopupMode::Session   => {
                                                if app.input.contains("delete") { "/session delete" }
                                                else { "/session continue" }
                                            },
                                            _                         => "",
                                        };
                                        if !filtered.is_empty() && app.selected_index < filtered.len() {
                                            let selection = &filtered[app.selected_index];
                                            let _ = tx_out.send(format!("{} {}", prefix, selection.id));
                                        }
                                        app.show_popup = false;
                                        app.popup_mode = app::PopupMode::None;
                                        app.popup_search_query.clear();
                                        app.input.clear();
                                    }
                                } else if let Some(msg) = app.submit_message() {
                                    let _ = tx_out.send(msg);
                                }
                            }
                            KeyCode::Char(c) => {
                                if app.show_popup {
                                    app.popup_search_query.push(c);
                                    app.input.push(c);
                                    app.set_popup_index(0);
                                } else {
                                    app.handle_char(c);
                                    let trigger = if app.input == "/" { Some(app::PopupMode::Commands) }
                                    else if app.input == "/world select " && !app.available_worlds.is_empty() { Some(app::PopupMode::World) }
                                    else if app.input == "/character select " && !app.available_characters.is_empty() { Some(app::PopupMode::Character) }
                                    else if app.input == "/model " && !app.available_models.is_empty() { Some(app::PopupMode::Model) }
                                    else if app.input == "/rules add " && !app.available_rules.is_empty() { Some(app::PopupMode::Rules) }
                                    else if (app.input == "/session continue " || app.input == "/session delete ") && !app.available_sessions.is_empty() { Some(app::PopupMode::Session) }
                                    else { None };

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
                                    app.input.pop();
                                    app.set_popup_index(0);
                                } else {
                                    app.handle_backspace();
                                }
                            }

                            KeyCode::Up => {
                                if app.show_popup { app.set_popup_index(app.selected_index.saturating_sub(1)); }
                                else { app.scroll = app.scroll.saturating_sub(1); }
                            }
                            KeyCode::Down => {
                                if app.show_popup {
                                    let max = app.get_filtered_items().len().saturating_sub(1);
                                    if app.selected_index < max { app.set_popup_index(app.selected_index + 1); }
                                } else { app.scroll = app.scroll.saturating_add(1); }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            // --- Animation / Tick ---
            _ = tokio::time::sleep(timeout).fuse() => {
                if app.is_typing {
                    app.spinner_frame = (app.spinner_frame + 1) % 10;
                }
                last_tick = std::time::Instant::now();
            }
        }
    }
}
