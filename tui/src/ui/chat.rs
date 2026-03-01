use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
    Frame,
};
use crate::app::{App, Role, SPINNER_FRAMES};
use crate::ui::{COLOR_AI, COLOR_SYSTEM, COLOR_ERROR, wrap_text};

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let chat_pane_width = area.width.saturating_sub(4) as usize; 
    let indent_width = 4usize;
    let body_width = chat_pane_width.saturating_sub(indent_width).max(20);

    let mut chat_lines: Vec<Line> = vec![Line::from("")]; 

    let push_message = |lines: &mut Vec<Line>, role: Role, text: &str| {
        let (icon, label, color) = match role {
            Role::Player => ("○", "YOU", Color::White),
            Role::World  => ("✦", "NARRATOR", COLOR_AI),
            Role::System => ("ℹ", "SYSTEM", COLOR_SYSTEM),
            Role::Error  => ("✖", "ERROR", COLOR_ERROR),
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{} ", icon), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(label, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ]));

        let wrapped = wrap_text(text, body_width);
        for segment in wrapped {
            lines.push(Line::from(vec![
                Span::raw(" ".repeat(indent_width)),
                Span::styled(segment, Style::default().fg(if role == Role::World { Color::White } else { color })),
            ]));
        }
        lines.push(Line::from("")); 
    };

    for msg in &app.messages {
        push_message(&mut chat_lines, msg.role.clone(), &msg.content);
    }

    if app.is_typing {
        let spinner = SPINNER_FRAMES[app.spinner_frame % SPINNER_FRAMES.len()];
        chat_lines.push(Line::from(vec![
            Span::styled("✦ ", Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD)),
            Span::styled("NARRATOR", Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  {} ", spinner), Style::default().fg(Color::Indexed(238))),
        ]));

        if !app.current_streaming_message.is_empty() {
            let wrapped = wrap_text(&app.current_streaming_message, body_width);
            for segment in wrapped {
                chat_lines.push(Line::from(vec![
                    Span::raw(" ".repeat(indent_width)),
                    Span::styled(segment, Style::default().fg(Color::White)),
                ]));
            }
        }
    }

    let chat_height = area.height;
    let total_lines = chat_lines.len() as u16;
    let max_scroll = total_lines.saturating_sub(chat_height);
    let safe_scroll = app.scroll.min(max_scroll);
    app.scroll = safe_scroll;

    f.render_widget(
        Paragraph::new(chat_lines)
            .block(Block::default().padding(Padding::horizontal(2)))
            .scroll((safe_scroll, 0)),
        area
    );
}
