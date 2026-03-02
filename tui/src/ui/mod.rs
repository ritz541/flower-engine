use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub mod header;
pub mod chat;
pub mod sidebar;
pub mod input;
pub mod popup;
pub mod checklist;

use crate::app::{App, Role};

pub const COLOR_AI: ratatui::style::Color = ratatui::style::Color::Indexed(211);
pub const COLOR_SYSTEM: ratatui::style::Color = ratatui::style::Color::Indexed(243);
pub const COLOR_ERROR: ratatui::style::Color = ratatui::style::Color::Indexed(160);
pub const COLOR_HEADER_BG: ratatui::style::Color = ratatui::style::Color::Indexed(232);
pub const COLOR_ACCENT: ratatui::style::Color = ratatui::style::Color::Indexed(218);
pub const COLOR_DIVIDER: ratatui::style::Color = ratatui::style::Color::Indexed(235);

pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 { return vec![text.to_string()]; }
    let mut lines: Vec<String> = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() { lines.push(String::new()); continue; }
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            if current.is_empty() { current.push_str(word); }
            else if current.len() + 1 + word.len() <= max_width {
                current.push(' '); current.push_str(word);
            } else {
                lines.push(current); current = word.to_string();
            }
        }
        lines.push(current);
    }
    if lines.is_empty() { lines.push(String::new()); }
    lines
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let mut constraints = vec![
        Constraint::Length(1), // Header
        Constraint::Min(1),    // Content
        Constraint::Length(1), // Divider
        Constraint::Length(3), // Input (3 lines max for multi-line input)
    ];
    if app.show_popup { constraints.push(Constraint::Length(6)); }

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(f.size());

    header::draw(f, app, root[0]);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(24)])
        .split(root[1]);

    let has_narrative = app.messages.iter().any(|m| m.role == Role::Player || m.role == Role::World);
    let session_active = !app.session_id.is_empty();

    if !has_narrative && !session_active && !app.is_typing {
        checklist::draw(f, app, middle[0]);
    } else {
        chat::draw(f, app, middle[0]);
    }

    sidebar::draw(f, app, middle[1]);
    
    let status_area = root[2];
    let status_line = Line::from(vec![
        Span::styled("──", Style::default().fg(COLOR_DIVIDER)),
        Span::styled(format!(" {} tokens ", app.total_tokens), Style::default().fg(Color::Indexed(238))),
        Span::styled("• ", Style::default().fg(COLOR_DIVIDER)),
        Span::styled(format!(" ${:.4} ", app.estimated_cost()), Style::default().fg(Color::Indexed(238))),
        Span::styled("• ", Style::default().fg(COLOR_DIVIDER)),
        Span::styled(format!(" {} elapsed ", app.session_elapsed()), Style::default().fg(Color::Indexed(238))),
        Span::styled("─".repeat(status_area.width as usize), Style::default().fg(COLOR_DIVIDER)),
    ]);
    f.render_widget(Paragraph::new(status_line), status_area);

    input::draw(f, app, root[3]);

    if app.show_popup {
        popup::draw(f, app, root[4]);
    }
}
