use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph},
    Frame,
};

use crate::app::{App, PopupMode, Role, SPINNER_FRAMES};

// ── Colors & Constants ────────────────────────────────────────────────────────

const COLOR_USER: Color = Color::White;
const COLOR_AI: Color = Color::Cyan;
const COLOR_SYSTEM: Color = Color::Indexed(244); // Gray
const COLOR_ERROR: Color = Color::Red;
const COLOR_HEADER_BG: Color = Color::Indexed(235); // Very dark gray
const COLOR_ACCENT: Color = Color::Indexed(141); // Purple-ish

// ── Helpers ───────────────────────────────────────────────────────────────────

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Word-wrap `text` into lines of at most `max_width` columns.
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }
    let mut lines: Vec<String> = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            if current.is_empty() {
                current.push_str(word);
            } else if current.len() + 1 + word.len() <= max_width {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current);
                current = word.to_string();
            }
        }
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

// ── Draw ──────────────────────────────────────────────────────────────────────

pub fn draw(f: &mut Frame, app: &mut App) {
    // ── Root layout ────────────────────────────────────────────────────────
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(1),    // Content
            Constraint::Length(1), // Divider/Status
            Constraint::Length(1), // Input
        ])
        .split(f.size());

    // ── HEADER ─────────────────────────────────────────────────────────────
    let model_short = app.active_model.split('/').last().unwrap_or(&app.active_model);
    let header_line = Line::from(vec![
        Span::styled(" 🌸 THE FLOWER ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(" │ ", Style::default().fg(Color::Indexed(239))),
        Span::styled(format!("{} ", app.world_id), Style::default().fg(COLOR_AI)),
        Span::styled("• ", Style::default().fg(Color::Indexed(239))),
        Span::styled(format!("{} ", app.character_id), Style::default().fg(Color::White)),
        Span::styled(" │ ", Style::default().fg(Color::Indexed(239))),
        Span::styled(format!("{} ", model_short), Style::default().fg(COLOR_ACCENT)),
        Span::styled(format!("({:.1} t/s) ", app.tps), Style::default().fg(Color::Indexed(240))),
    ]);
    f.render_widget(
        Paragraph::new(header_line).style(Style::default().bg(COLOR_HEADER_BG)),
        root[0]
    );

    // ── CHAT CONTENT ───────────────────────────────────────────────────────
    let chat_pane_width = root[1].width.saturating_sub(4) as usize; // padding
    let indent_width = 4usize;
    let body_width = chat_pane_width.saturating_sub(indent_width).max(20);

    let mut chat_lines: Vec<Line> = vec![Line::from("")]; // Top padding

    // Helper to push stylized messages
    let mut push_message = |lines: &mut Vec<Line>, role: Role, text: &str| {
        let (icon, label, color) = match role {
            Role::Player => ("○", "YOU", COLOR_USER),
            Role::World  => ("✦", "NARRATOR", COLOR_AI),
            Role::System => ("ℹ", "SYSTEM", COLOR_SYSTEM),
            Role::Error  => ("✖", "ERROR", COLOR_ERROR),
        };

        // Header line for the message
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", icon), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(label, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ]));

        // Body text
        let wrapped = wrap_text(text, body_width);
        for segment in wrapped {
            lines.push(Line::from(vec![
                Span::raw(" ".repeat(indent_width)),
                Span::styled(segment, Style::default().fg(if role == Role::World { Color::White } else { color })),
            ]));
        }
        lines.push(Line::from("")); // Gap
    };

    // Render all messages
    for msg in &app.messages.clone() {
        push_message(&mut chat_lines, msg.role.clone(), &msg.content);
    }

    // Live streaming message
    if app.is_typing {
        let spinner = SPINNER_FRAMES[app.spinner_frame % SPINNER_FRAMES.len()];
        let (icon, label, color) = ("✦", "NARRATOR", COLOR_AI);
        
        chat_lines.push(Line::from(vec![
            Span::styled(format!("{} ", icon), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(label, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  {} ", spinner), Style::default().fg(Color::Indexed(240))),
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

    // Scroll management
    let chat_height = root[1].height;
    let total_lines = chat_lines.len() as u16;
    let max_scroll = total_lines.saturating_sub(chat_height);
    let safe_scroll = app.scroll.min(max_scroll);
    app.scroll = safe_scroll;

    f.render_widget(
        Paragraph::new(chat_lines)
            .block(Block::default().padding(Padding::horizontal(2)))
            .scroll((safe_scroll, 0)),
        root[1]
    );

    // ── STATUS / DIVIDER ──────────────────────────────────────────────────
    let cost_str = format!("${:.4}", app.estimated_cost());
    let token_str = if app.total_tokens >= 1000 {
        format!("{:.1}k", app.total_tokens as f64 / 1000.0)
    } else {
        app.total_tokens.to_string()
    };

    let status_line = Line::from(vec![
        Span::styled("─".repeat(2), Style::default().fg(Color::Indexed(239))),
        Span::styled(format!(" {} tokens ", token_str), Style::default().fg(Color::Indexed(244))),
        Span::styled("• ", Style::default().fg(Color::Indexed(239))),
        Span::styled(format!(" {} cost ", cost_str), Style::default().fg(Color::Indexed(244))),
        Span::styled("• ", Style::default().fg(Color::Indexed(239))),
        Span::styled(format!(" {} elapsed ", app.session_elapsed()), Style::default().fg(Color::Indexed(244))),
        Span::styled("─".repeat(root[2].width as usize), Style::default().fg(Color::Indexed(239))),
    ]);
    f.render_widget(Paragraph::new(status_line), root[2]);

    // ── INPUT BAR ─────────────────────────────────────────────────────────
    let cursor = if app.cursor_state { "▋" } else { " " };
    let prompt = if app.is_typing {
        Line::from(vec![
            Span::styled("  ✦ ", Style::default().fg(COLOR_AI)),
            Span::styled("Narrating...", Style::default().fg(Color::Indexed(240)).add_modifier(Modifier::ITALIC)),
            Span::styled("  (Esc to stop)", Style::default().fg(Color::Indexed(236))),
        ])
    } else {
        let input_text = if app.input.is_empty() {
            Span::styled("How do you respond? (/ for commands)", Style::default().fg(Color::Indexed(239)))
        } else {
            Span::styled(app.input.clone(), Style::default().fg(Color::White))
        };

        let hint_span = if !app.command_hint.is_empty() {
            Span::styled(app.command_hint.clone(), Style::default().fg(Color::Indexed(237)))
        } else {
            Span::raw("")
        };

        Line::from(vec![
            Span::styled("  › ", Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD)),
            input_text,
            hint_span,
            Span::styled(cursor, Style::default().fg(COLOR_AI)),
        ])
    };
    f.render_widget(Paragraph::new(prompt), root[3]);

    // ── COMMAND POPUP ─────────────────────────────────────────────────────
    if app.show_popup {
        let popup_height = 10;
        let popup_area = Rect::new(
            root[1].x + 2,
            root[1].y + root[1].height.saturating_sub(popup_height),
            root[1].width.saturating_sub(4),
            popup_height.min(root[1].height),
        );
        f.render_widget(Clear, popup_area);

        let popup_title = match app.popup_mode {
            PopupMode::World     => " WORLD ",
            PopupMode::Character => " CHARACTER ",
            PopupMode::Model     => " MODEL ",
            PopupMode::Rules     => " RULES ",
            _                    => " SELECT ",
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Indexed(239)))
            .title(Span::styled(popup_title, Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD)))
            .padding(Padding::horizontal(1));
        
        let inner_area = block.inner(popup_area);
        f.render_widget(block, popup_area);
        
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(inner_area);

        // Search Bar
        let search_cursor = if app.cursor_state { "▋" } else { " " };
        let search_text = if app.popup_search_query.is_empty() {
            Span::styled("Search...", Style::default().fg(Color::Indexed(239)))
        } else {
            Span::styled(app.popup_search_query.clone(), Style::default().fg(Color::White))
        };
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(" 🔍 ", Style::default().fg(Color::Indexed(240))),
                search_text,
                Span::styled(search_cursor, Style::default().fg(COLOR_AI)),
            ])),
            popup_layout[0]
        );

        // Results
        let items = app.get_filtered_items();
        let list_items: Vec<ListItem> = if items.is_empty() {
            vec![ListItem::new("   No results found").style(Style::default().fg(Color::Indexed(239)))]
        } else {
            items.iter().enumerate().map(|(i, entity)| {
                let content = if i == app.selected_index {
                    format!(" ❯ {}  ", entity.name)
                } else {
                    format!("   {}  ", entity.name)
                };
                let style = if i == app.selected_index {
                    Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD).bg(Color::Indexed(236))
                } else {
                    Style::default().fg(Color::Indexed(244))
                };
                ListItem::new(content).style(style)
            }).collect()
        };

        f.render_widget(List::new(list_items), popup_layout[1]);
    }
}
