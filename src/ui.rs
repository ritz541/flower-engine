use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph},
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
    let mut constraints = vec![
        Constraint::Length(1), // Header
        Constraint::Min(1),    // Content
        Constraint::Length(1), // Divider/Status
        Constraint::Length(1), // Input
    ];
    if app.show_popup {
        constraints.push(Constraint::Length(6)); // Suggestions Area
    }

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
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

    // ── MIDDLE: SIDEBAR │ CHAT ───────────────────────────────────────────
    let content_area = root[1];
    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),     // Chat
            Constraint::Length(24), // Sidebar
        ])
        .split(content_area);

    let chat_area = middle[0];
    let sidebar_area = middle[1];

    // Background for chat
    f.render_widget(Block::default().bg(Color::Indexed(234)), chat_area);

    // ── CHAT CONTENT ───────────────────────────────────────────────────────
    let chat_pane_width = chat_area.width.saturating_sub(4) as usize; 
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
        chat_area
    );

    // ── SIDEBAR ────────────────────────────────────────────────────────────
    let mut sidebar_lines = vec![
        Line::from(""),
        Line::from(Span::styled(" STATUS", Style::default().fg(Color::Indexed(240)).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("  {} ", app.status), Style::default().fg(COLOR_AI))),
        Line::from(""),
        Line::from(Span::styled(" WORLD", Style::default().fg(Color::Indexed(240)).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("  {} ", app.world_id), Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled(" CHARACTER", Style::default().fg(Color::Indexed(240)).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("  {} ", app.character_id), Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled(" ACTIVE RULES", Style::default().fg(Color::Indexed(240)).add_modifier(Modifier::BOLD))),
    ];

    if app.active_rules.is_empty() {
        sidebar_lines.push(Line::from(Span::styled("  (none)", Style::default().fg(Color::Indexed(237)))));
    } else {
        for rule in &app.active_rules {
            sidebar_lines.push(Line::from(Span::styled(format!("  • {} ", rule), Style::default().fg(COLOR_ACCENT))));
        }
    }

    f.render_widget(
        Paragraph::new(sidebar_lines)
            .block(Block::default().borders(Borders::LEFT).border_style(Style::default().fg(Color::Indexed(236)))),
        sidebar_area
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
    let cursor = "█"; // STEADY BOX CURSOR
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

    // ── COMMAND SUGGESTIONS (UNDER INPUT) ─────────────────────────────────
    if app.show_popup {
        let area = root[4];
        
        let popup_title = match app.popup_mode {
            PopupMode::World     => " WORLD ",
            PopupMode::Character => " CHARACTER ",
            PopupMode::Model     => " MODEL ",
            PopupMode::Rules     => " RULES ",
            _                    => " SELECT ",
        };

        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::Indexed(237)))
            .title(Span::styled(format!(" {} ", popup_title), Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD)))
            .padding(Padding::horizontal(2));
        
        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let items = app.get_filtered_items();
        let list_items: Vec<ListItem> = if items.is_empty() {
            vec![ListItem::new("   No matches found").style(Style::default().fg(Color::Indexed(239)))]
        } else {
            items.iter().enumerate().map(|(i, entity)| {
                let is_selected = i == app.selected_index;
                let content = if is_selected {
                    format!(" ❯ {}  ", entity.name)
                } else {
                    format!("   {}  ", entity.name)
                };
                let style = if is_selected {
                    Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Indexed(244))
                };
                ListItem::new(content).style(style)
            }).collect()
        };

        f.render_stateful_widget(
            List::new(list_items).highlight_style(Style::default().bg(Color::Indexed(236))),
            inner_area,
            &mut app.popup_state
        );
    }
}
