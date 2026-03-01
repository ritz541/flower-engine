mod app;
mod models;
mod ui;
mod ws;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io, time::{Duration, Instant}};

use app::App;
use models::WsMessage;
use tokio::sync::mpsc;

const TICK_RATE: Duration = Duration::from_millis(500); // For cursor blinking

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
                    app.finish_stream();
                    app.status = "Idle".to_string();
                }
                "error" => {
                    app.add_system_message(format!("Server Error: {}", msg.payload.content));
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
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Enter => {
                        if let Some(msg) = app.submit_message() {
                            // If local command `/` just push to WS directly to be handled
                            let _ = tx_out.send(msg);
                        }
                    }
                    KeyCode::Char(c) => app.handle_char(c),
                    KeyCode::Backspace => app.handle_backspace(),
                    KeyCode::Up => {
                        app.scroll = app.scroll.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        app.scroll = app.scroll.saturating_add(1);
                    }
                    _ => {}
                }
            }
        }
        
        // --- TICK UPDATE (Cursor Animation) ---
        if last_tick.elapsed() >= TICK_RATE {
            app.cursor_state = !app.cursor_state; // Toggle cursor blink
            last_tick = Instant::now();
        }
    }
}
