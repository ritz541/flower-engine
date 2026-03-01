use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem, Wrap, Padding},
    Frame,
};

use crate::app::{App, Role};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Status Bar Header
            Constraint::Min(1),    // Main content
            Constraint::Length(3), // Input
        ])
        .split(f.size());

    // --- Pro Status Bar ---
    let status_text = format!(
        " System: {} | Model: {} | TPS: {:.1} ",
        app.status, app.active_model, app.tps
    );
    let header = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL).title(" Roleplay Engine Core "))
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(header, chunks[0]);

    // --- Middle Section (Sidebar + Chat) ---
    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Sidebar
            Constraint::Percentage(80), // Chat
        ])
        .split(chunks[1]);

    // Sidebar
    let world_info = ListItem::new(format!("🌍 World:\n  {}", app.world_id));
    let char_info = ListItem::new(format!("👤 Character:\n  {}", app.character_id));
    let sidebar_items = vec![world_info, ListItem::new(""), char_info];
    
    let sidebar = List::new(sidebar_items)
        .block(Block::default().borders(Borders::ALL).title(" State ").padding(Padding::uniform(1)))
        .style(Style::default().fg(Color::Indexed(14))); // Soft Cyan
    f.render_widget(sidebar, middle_chunks[0]);

    // Chat (Adaptive Buffering)
    let mut chat_lines = Vec::new();
    
    for msg in &app.messages {
        let (prefix, style) = match msg.role {
            Role::Player => ("You: ", Style::default().fg(Color::White)),
            Role::Ai => ("AI: ", Style::default().fg(Color::Indexed(14))), // Soft Cyan
            Role::System => ("System: ", Style::default().fg(Color::DarkGray)),
        };
        
        chat_lines.push(Line::from(Span::styled(format!("{}{}", prefix, msg.content), style)));
        chat_lines.push(Line::from("")); // Spacing
    }

    if app.is_typing {
        let cursor = if app.cursor_state { "|" } else { " " };
        let stream_text = format!("AI: {}{}", app.current_streaming_message, cursor);
        chat_lines.push(Line::from(Span::styled(stream_text, Style::default().fg(Color::Indexed(14)))));
    }

    let chat_paragraph = Paragraph::new(chat_lines)
        .block(Block::default().borders(Borders::ALL).title(" Output Window ").padding(Padding::uniform(1)))
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0));
    
    f.render_widget(chat_paragraph, middle_chunks[1]);

    // --- Input Footer ---
    let input_style = if app.is_typing {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input_text = if app.is_typing {
        "Waiting for sequence completion..."
    } else {
        &app.input
    };

    let input_paragraph = Paragraph::new(input_text)
        .style(input_style)
        .block(Block::default().borders(Borders::ALL).title(" Input (/model, /world, or message) ").padding(Padding::horizontal(1)));
    f.render_widget(input_paragraph, chunks[2]);
}
