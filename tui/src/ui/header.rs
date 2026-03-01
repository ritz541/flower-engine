use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::App;
use crate::ui::{COLOR_AI, COLOR_DIVIDER, COLOR_ACCENT, COLOR_HEADER_BG};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let model_short = app.active_model.split('/').last().unwrap_or(&app.active_model);
    let header_line = Line::from(vec![
        Span::styled(" 🌸 THE FLOWER ", Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD)),
        Span::styled(" │ ", Style::default().fg(COLOR_DIVIDER)),
        Span::styled(format!("{} ", app.world_id), Style::default().fg(ratatui::style::Color::White)),
        Span::styled("• ", Style::default().fg(COLOR_DIVIDER)),
        Span::styled(format!("{} ", app.character_id), Style::default().fg(ratatui::style::Color::Indexed(248))),
        Span::styled(" │ ", Style::default().fg(COLOR_DIVIDER)),
        Span::styled(format!("{} ", model_short), Style::default().fg(COLOR_ACCENT)),
        Span::styled(format!("({:.1} t/s) ", app.tps), Style::default().fg(ratatui::style::Color::Indexed(238))),
    ]);
    f.render_widget(
        Paragraph::new(header_line).style(Style::default().bg(COLOR_HEADER_BG)),
        area
    );
}
