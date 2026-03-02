use crate::app::App;
use crate::ui::{COLOR_ACCENT, COLOR_AI};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let mut sidebar_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            " STATUS",
            Style::default()
                .fg(Color::Indexed(237))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {} ", app.status),
            Style::default().fg(COLOR_AI),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " WORLD",
            Style::default()
                .fg(Color::Indexed(237))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {} ", app.world_id),
            Style::default().fg(COLOR_AI),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " CHARACTER",
            Style::default()
                .fg(Color::Indexed(237))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {} ", app.character_id),
            Style::default().fg(COLOR_AI),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " ACTIVE RULES",
            Style::default()
                .fg(Color::Indexed(237))
                .add_modifier(Modifier::BOLD),
        )),
    ];

    if app.active_rules.is_empty() {
        sidebar_lines.push(Line::from(Span::styled(
            "  (none)",
            Style::default().fg(Color::Indexed(234)),
        )));
    } else {
        for rule in &app.active_rules {
            sidebar_lines.push(Line::from(Span::styled(
                format!("  • {} ", rule),
                Style::default().fg(COLOR_ACCENT),
            )));
        }
    }

    f.render_widget(
        Paragraph::new(sidebar_lines).block(Block::default().borders(Borders::NONE)),
        area,
    );
}
