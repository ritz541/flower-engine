use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::COLOR_AI;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let mut checklist = vec![
        Line::from(""),
        Line::from(Span::styled("  WELCOME, WANDERER", Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("  To begin your story, please prepare the stage:", Style::default().fg(Color::Indexed(240)))),
        Line::from(""),
    ];

    let mut add_item = |label: &str, done: bool, cmd: &str, val: &str| {
        let (icon, color, sub) = if done {
            ("  ✓", COLOR_AI, format!(" (Selected: {})", val))
        } else {
            ("  ○", Color::Indexed(237), format!(" (Type {})", cmd))
        };
        checklist.push(Line::from(vec![
            Span::styled(format!("{} ", icon), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<14}", label), Style::default().fg(if done { Color::White } else { Color::Indexed(238) })),
            Span::styled(sub, Style::default().fg(Color::Indexed(235))),
        ]));
    };

    let world_done = app.world_id != "Connecting..." && !app.world_id.is_empty();
    let char_done = app.character_id != "Connecting..." && app.character_id != "Wanderer" && !app.character_id.is_empty();
    
    add_item("World", world_done, "/world select", &app.world_id);
    add_item("Character", char_done, "/character select", &app.character_id);
    add_item("AI Model", app.model_confirmed, "/model", &app.active_model);
    add_item("Modules (Opt)", !app.active_modules.is_empty(), "/module add", "Optional");
    add_item("Session", !app.session_id.is_empty(), "/session new", &app.session_id);
    if !app.session_id.is_empty() {
        checklist.push(Line::from(""));
        checklist.push(Line::from(Span::styled("  ✦ The stage is set. Send a message to begin.", Style::default().fg(COLOR_AI).add_modifier(Modifier::ITALIC))));
    }

    // --- MINI SYSTEM LOG ---
    let sys_msgs: Vec<_> = app.messages.iter()
        .filter(|m| m.role == crate::app::Role::System || m.role == crate::app::Role::Error)
        .rev()
        .take(2)
        .collect();

    if !sys_msgs.is_empty() {
        checklist.push(Line::from(""));
        for msg in sys_msgs.iter().rev() {
            let color = if msg.role == crate::app::Role::Error { crate::ui::COLOR_ERROR } else { Color::Indexed(239) };
            checklist.push(Line::from(vec![
                Span::styled("  ! ", Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(&msg.content, Style::default().fg(color)),
            ]));
        }
    }

    f.render_widget(
        Paragraph::new(checklist)
            .block(Block::default().padding(Padding::vertical((area.height / 4).saturating_sub(2)))),
        area
    );
}
