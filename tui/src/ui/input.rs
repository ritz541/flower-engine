use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::COLOR_AI;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let cursor = "█"; 
    let prompt = if app.is_typing {
        Line::from(vec![
            Span::styled("  ✦ ", Style::default().fg(COLOR_AI)),
            Span::styled("Narrating...", Style::default().fg(Color::Indexed(236)).add_modifier(Modifier::ITALIC)),
        ])
    } else {
        let (input_span, cursor_after) = if app.input.is_empty() {
            (
                Span::styled("How do you respond? (/ for commands)", Style::default().fg(Color::Indexed(236))),
                false
            )
        } else {
            (
                Span::styled(app.input.clone(), Style::default().fg(Color::White)),
                true
            )
        };

        let hint_span = if !app.command_hint.is_empty() {
            Span::styled(app.command_hint.clone(), Style::default().fg(Color::Indexed(234)))
        } else {
            Span::raw("")
        };

        if cursor_after {
            Line::from(vec![
                Span::styled("  › ", Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD)),
                input_span,
                Span::styled(cursor, Style::default().fg(COLOR_AI)),
                hint_span,
            ])
        } else {
            Line::from(vec![
                Span::styled("  › ", Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD)),
                Span::styled(cursor, Style::default().fg(COLOR_AI)),
                input_span,
                hint_span,
            ])
        }
    };
    
    f.render_widget(
        Paragraph::new(prompt).block(Block::default().bg(Color::Indexed(233))),
        area
    );
}
