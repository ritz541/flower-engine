use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::{COLOR_AI, COLOR_ACCENT};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let mut sidebar_lines = vec![
        Line::from(""),
        Line::from(Span::styled(" STATUS", Style::default().fg(Color::Indexed(237)).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("  {} ", app.status), Style::default().fg(COLOR_AI))),
        Line::from(""),
        Line::from(Span::styled(" WORLD", Style::default().fg(Color::Indexed(237)).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("  {} ", app.world_id), Style::default().fg(Color::Indexed(244)))),
        Line::from(""),
        Line::from(Span::styled(" CHARACTER", Style::default().fg(Color::Indexed(237)).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("  {} ", app.character_id), Style::default().fg(Color::Indexed(244)))),
        Line::from(""),
        Line::from(Span::styled(" SKILLS", Style::default().fg(Color::Indexed(237)).add_modifier(Modifier::BOLD))),
    ];

    if app.active_skills.is_empty() {
        sidebar_lines.push(Line::from(Span::styled("  (none)", Style::default().fg(Color::Indexed(234)))));
    } else {
        for skill in &app.active_skills {
            sidebar_lines.push(Line::from(Span::styled(format!("  • {} ", skill), Style::default().fg(COLOR_AI))));
        }
    }

    sidebar_lines.extend([
        Line::from(""),
        Line::from(Span::styled(" MODULES", Style::default().fg(Color::Indexed(237)).add_modifier(Modifier::BOLD))),
    ]);

    if app.active_modules.is_empty() {
        sidebar_lines.push(Line::from(Span::styled("  (none)", Style::default().fg(Color::Indexed(234)))));
    } else {
        for mod_id in &app.active_modules {
            sidebar_lines.push(Line::from(Span::styled(format!("  • {} ", mod_id), Style::default().fg(COLOR_AI))));
        }
    }

    sidebar_lines.extend([
        Line::from(""),
        Line::from(Span::styled(" ACTIVE RULES", Style::default().fg(Color::Indexed(237)).add_modifier(Modifier::BOLD))),
    ]);

    if app.active_rules.is_empty() {
        sidebar_lines.push(Line::from(Span::styled("  (none)", Style::default().fg(Color::Indexed(234)))));
    } else {
        for rule in &app.active_rules {
            sidebar_lines.push(Line::from(Span::styled(format!("  • {} ", rule), Style::default().fg(COLOR_ACCENT))));
        }
    }

    f.render_widget(
        Paragraph::new(sidebar_lines).block(Block::default().borders(Borders::NONE)),
        area
    );
}
