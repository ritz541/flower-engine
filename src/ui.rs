use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph},
    Frame,
};

use crate::app::{App, PopupMode, Role, SPINNER_FRAMES};

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
            Constraint::Length(1), // header
            Constraint::Min(1),    // content
            Constraint::Length(1), // divider
            Constraint::Length(2), // input
        ])
        .split(f.size());

    // ── HEADER ─────────────────────────────────────────────────────────────
    let model_short = app.active_model.split('/').last().unwrap_or(&app.active_model);
    let header_line = Line::from(vec![
        Span::styled(" 🌸 The Flower  ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("  🌍 {}  ", app.world_id), Style::default().fg(Color::White)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("  👤 {}  ", app.character_id), Style::default().fg(Color::White)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("  ⚡ {}  ", model_short), Style::default().fg(Color::Indexed(141))),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("  {:.1} t/s  ", app.tps), Style::default().fg(Color::Green)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("  {} msgs  ", app.message_count), Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(header_line), root[0]);

    // ── MIDDLE: sidebar │ chat ────────────────────────────────────────────
    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(root[1]);

    // ── SIDEBAR ────────────────────────────────────────────────────────────
    let token_str = if app.total_tokens >= 1000 {
        format!("{:.1}k", app.total_tokens as f64 / 1000.0)
    } else {
        app.total_tokens.to_string()
    };
    let cost_str = format!("${:.4}", app.estimated_cost());

    let mut sidebar_lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(" SESSION", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("  World   {}", app.world_id),        Style::default().fg(Color::Cyan))),
        Line::from(Span::styled(format!("  Char    {}", app.character_id),     Style::default().fg(Color::White))),
        Line::from(Span::styled(format!("  Model   {}", model_short),          Style::default().fg(Color::Indexed(141)))),
        Line::from(""),
        Line::from(Span::styled(" RULES", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
    ];
    if app.active_rules.is_empty() {
        sidebar_lines.push(Line::from(Span::styled("  (none)", Style::default().fg(Color::DarkGray))));
    } else {
        for r in &app.active_rules {
            sidebar_lines.push(Line::from(Span::styled(format!("  • {}", r), Style::default().fg(Color::Yellow))));
        }
    }
    sidebar_lines.extend([
        Line::from(""),
        Line::from(Span::styled(" STATS", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("  TPS    {:.1}", app.tps),       Style::default().fg(Color::Green))),
        Line::from(Span::styled(format!("  Tokens {}", token_str),         Style::default().fg(Color::White))),
        Line::from(Span::styled(format!("  Msgs   {}", app.message_count), Style::default().fg(Color::White))),
        Line::from(Span::styled(format!("  Cost   {}", cost_str),          Style::default().fg(Color::Indexed(208)))),
        Line::from(Span::styled(format!("  Time   {}", app.session_elapsed()), Style::default().fg(Color::DarkGray))),
    ]);
    f.render_widget(Paragraph::new(sidebar_lines).block(Block::default().borders(Borders::NONE)), middle[0]);

    // Thin vertical divider
    let div_height = middle[1].height as usize;
    let div_lines: Vec<Line> = (0..div_height)
        .map(|_| Line::from(Span::styled("│", Style::default().fg(Color::DarkGray))))
        .collect();
    f.render_widget(Paragraph::new(div_lines), middle[1]);

    // ── CHAT PANE (pre-wrapped, no Ratatui Wrap) ──────────────────────────

    // Body text width: pane width minus 2 padding, minus 8 chars for tag + spaces
    let chat_pane_width = middle[2].width.saturating_sub(2) as usize; // inner after padding
    let tag_width = 8usize; // " GM   " (6) + "  " (2)
    let body_width = chat_pane_width.saturating_sub(tag_width).max(20);

    let mut chat_lines: Vec<Line> = vec![Line::from("")]; // top padding

    // Helper closure: add a full pre-wrapped message to chat_lines
    let push_message = |chat_lines: &mut Vec<Line>,
                             tag: &'static str,
                             tag_bg: Color,
                             body_color: Color,
                             text: &str| {
        let wrapped = wrap_text(text, body_width);
        let mut is_first = true;
        for segment in wrapped {
            let line = if is_first {
                is_first = false;
                Line::from(vec![
                    Span::styled(tag, Style::default().bg(tag_bg).fg(Color::Black).add_modifier(Modifier::BOLD)),
                    Span::raw("  "),
                    Span::styled(segment, Style::default().fg(body_color)),
                ])
            } else {
                Line::from(vec![
                    Span::raw("        "), // same width as tag+"  "
                    Span::styled(segment, Style::default().fg(body_color)),
                ])
            };
            chat_lines.push(line);
        }
        chat_lines.push(Line::from("")); // gap after message
    };

    // Build all completed messages
    for msg in &app.messages.clone() {
        let (tag, tag_bg, body_color): (&'static str, Color, Color) = match msg.role {
            Role::Player => (" You  ", Color::White,        Color::White),
            Role::World  => (" GM   ", Color::Cyan,         Color::Indexed(14)),
            Role::System => (" Info ", Color::Indexed(220), Color::Indexed(220)),
            Role::Error  => (" Err  ", Color::Red,          Color::Red),
        };
        push_message(&mut chat_lines, tag, tag_bg, body_color, &msg.content.clone());
    }

    // Live streaming line
    if app.is_typing {
        let spinner = SPINNER_FRAMES[app.spinner_frame % SPINNER_FRAMES.len()];
        if app.current_streaming_message.is_empty() {
            chat_lines.push(Line::from(vec![
                Span::styled(" GM   ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(spinner.to_string(), Style::default().fg(Color::DarkGray)),
            ]));
        } else {
            let streaming_text = app.current_streaming_message.clone();
            let wrapped = wrap_text(&streaming_text, body_width);
            let mut first = true;
            for segment in wrapped {
                let line = if first {
                    first = false;
                    Line::from(vec![
                        Span::styled(" GM   ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)),
                        Span::raw("  "),
                        Span::styled(segment, Style::default().fg(Color::Indexed(14))),
                    ])
                } else {
                    Line::from(vec![
                        Span::raw("        "),
                        Span::styled(segment, Style::default().fg(Color::Indexed(14))),
                    ])
                };
                chat_lines.push(line);
            }
        }
    }

    // Scroll clamping with accurate line count (no Ratatui Wrap)
    let chat_height = middle[2].height;
    let total_lines = chat_lines.len() as u16;
    let max_scroll = total_lines.saturating_sub(chat_height);
    let safe_scroll = app.scroll.min(max_scroll);
    app.scroll = safe_scroll; // write back so Up/Down work from correct position

    let chat = Paragraph::new(chat_lines)
        .block(Block::default().borders(Borders::NONE).padding(Padding::horizontal(1)))
        .scroll((safe_scroll, 0));
    f.render_widget(chat, middle[2]);

    // ── HORIZONTAL DIVIDER ────────────────────────────────────────────────
    let divider = Paragraph::new(Line::from(Span::styled(
        "─".repeat(root[2].width as usize),
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(divider, root[2]);

    // ── INPUT BAR ─────────────────────────────────────────────────────────
    let cursor = if app.cursor_state { "▌" } else { " " };
    let input_line = if app.is_typing {
        let spinner = SPINNER_FRAMES[app.spinner_frame % SPINNER_FRAMES.len()];
        Line::from(vec![
            Span::styled(format!("  {} ", spinner), Style::default().fg(Color::DarkGray)),
            Span::styled("Narrating…", Style::default().fg(Color::DarkGray)),
            Span::styled("   Esc to stop", Style::default().fg(Color::Indexed(238))),
        ])
    } else if app.input.is_empty() {
        Line::from(vec![
            Span::styled("  › ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "/world  /character  /model  /rules — or type your action",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(cursor, Style::default().fg(Color::Cyan)),
        ])
    } else {
        Line::from(vec![
            Span::styled("  › ", Style::default().fg(Color::Cyan)),
            Span::styled(app.input.clone(), Style::default().fg(Color::White)),
            Span::styled(cursor, Style::default().fg(Color::Cyan)),
        ])
    };
    f.render_widget(Paragraph::new(input_line), root[3]);

    // ── SEARCH POPUP ──────────────────────────────────────────────────────
    if app.show_popup {
        let area = centered_rect(62, 50, f.size());
        f.render_widget(Clear, area);

        let popup_title = match app.popup_mode {
            PopupMode::World     => " 🌍 Select World ",
            PopupMode::Character => " 👤 Select Character ",
            PopupMode::Model     => " ⚡ Select Model ",
            PopupMode::Rules     => " 📜 Activate Rule ",
            _                    => " Select ",
        };

        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        let search_cursor = if app.cursor_state { "▌" } else { " " };
        let search_bar = Paragraph::new(format!("  🔍 {}{}", app.popup_search_query, search_cursor))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(popup_title)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::White));
        f.render_widget(search_bar, popup_layout[0]);

        let items = app.get_filtered_items();
        let selected = app.selected_index;
        let list_items: Vec<ListItem> = if items.is_empty() {
            vec![ListItem::new("  No matches").style(Style::default().fg(Color::DarkGray))]
        } else {
            items.iter().enumerate().map(|(i, entity)| {
                let content = format!("  {}  ({})", entity.name, entity.id);
                if i == selected {
                    ListItem::new(content).style(
                        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
                    )
                } else {
                    ListItem::new(content).style(Style::default().fg(Color::White))
                }
            }).collect()
        };

        let popup_list = List::new(list_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .padding(Padding::horizontal(1)),
        );
        f.render_widget(popup_list, popup_layout[1]);
    }
}
