use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph},
    Frame,
};

use crate::app::{App, PopupMode, Role, SPINNER_FRAMES};

// ── Colors & Constants ────────────────────────────────────────────────────────

const COLOR_AI: Color = Color::Indexed(211);     // Soft Rose/Pink
const COLOR_SYSTEM: Color = Color::Indexed(243); // Dimmed Gray
const COLOR_ERROR: Color = Color::Indexed(160);  // Soft Red
const COLOR_HEADER_BG: Color = Color::Indexed(232); // Deep Black-Gray
const COLOR_ACCENT: Color = Color::Indexed(218); // Lighter Pink
const COLOR_DIVIDER: Color = Color::Indexed(235); // Subtle separator

// ── Helpers ───────────────────────────────────────────────────────────────────

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
        Span::styled(" 🌸 THE FLOWER ", Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD)),
        Span::styled(" │ ", Style::default().fg(COLOR_DIVIDER)),
        Span::styled(format!("{} ", app.world_id), Style::default().fg(Color::White)),
        Span::styled("• ", Style::default().fg(COLOR_DIVIDER)),
        Span::styled(format!("{} ", app.character_id), Style::default().fg(Color::Indexed(248))),
        Span::styled(" │ ", Style::default().fg(COLOR_DIVIDER)),
        Span::styled(format!("{} ", model_short), Style::default().fg(COLOR_ACCENT)),
        Span::styled(format!("({:.1} t/s) ", app.tps), Style::default().fg(Color::Indexed(238))),
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

    // ── CHAT CONTENT ───────────────────────────────────────────────────────
    let chat_pane_width = chat_area.width.saturating_sub(4) as usize; 
    let indent_width = 4usize;
    let body_width = chat_pane_width.saturating_sub(indent_width).max(20);

    let has_narrative = app.messages.iter().any(|m| m.role == Role::Player || m.role == Role::World);

    if !has_narrative && !app.is_typing {
        // --- ONBOARDING CHECKLIST ---
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

        let world_done = app.world_id != "Connecting..." && app.world_id != "Unknown World" && !app.world_id.is_empty();
        let char_done = app.character_id != "Connecting..." && app.character_id != "Wanderer" && !app.character_id.is_empty();
        
        add_item("World", world_done, "/world select", &app.world_id);
        add_item("Character", char_done, "/character select", &app.character_id);
        add_item("AI Model", app.model_confirmed, "/model", &app.active_model);
        add_item("Session", !app.session_id.is_empty(), "/session new", &app.session_id);

        if !app.session_id.is_empty() {
            checklist.push(Line::from(""));
            checklist.push(Line::from(Span::styled("  ✦ The stage is set. Send a message to begin.", Style::default().fg(COLOR_AI).add_modifier(Modifier::ITALIC))));
        }

        f.render_widget(
            Paragraph::new(checklist)
                .block(Block::default().padding(Padding::vertical((chat_area.height / 4).saturating_sub(2)))),
            chat_area
        );
    } else {
let mut chat_lines: Vec<Line> = vec![Line::from("")]; // Top padding


    // Helper to push stylized messages
    let push_message = |lines: &mut Vec<Line>, role: Role, text: &str| {
        let (icon, label, color) = match role {
            Role::Player => ("○", "YOU", Color::White),
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
    }

    // ── SIDEBAR ────────────────────────────────────────────────────────────

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
        Paragraph::new(sidebar_lines)
            .block(Block::default().borders(Borders::NONE)),
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
        Span::styled("──", Style::default().fg(COLOR_DIVIDER)),
        Span::styled(format!(" {} tokens ", token_str), Style::default().fg(Color::Indexed(238))),
        Span::styled("• ", Style::default().fg(COLOR_DIVIDER)),
        Span::styled(format!(" {} cost ", cost_str), Style::default().fg(Color::Indexed(238))),
        Span::styled("• ", Style::default().fg(COLOR_DIVIDER)),
        Span::styled(format!(" {} elapsed ", app.session_elapsed()), Style::default().fg(Color::Indexed(238))),
        Span::styled("─".repeat(root[2].width as usize), Style::default().fg(COLOR_DIVIDER)),
    ]);
    f.render_widget(Paragraph::new(status_line), root[2]);

    // ── INPUT BAR ─────────────────────────────────────────────────────────
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
        Paragraph::new(prompt)
            .block(Block::default().bg(Color::Indexed(233))),
        root[3]
    );

    // ── COMMAND SUGGESTIONS (UNDER INPUT) ─────────────────────────────────
    if app.show_popup {
        let area = root[4];
        
        let popup_title = match app.popup_mode {
            PopupMode::World     => " WORLD ",
            PopupMode::Character => " CHARACTER ",
            PopupMode::Model     => " MODEL ",
            PopupMode::Rules     => " RULES ",
            PopupMode::Session   => " SESSION ",
            PopupMode::Skills    => " SKILLS ",
            PopupMode::Commands  => " COMMANDS ",
            _                    => " SELECT ",
        };

        let block = Block::default()
            .borders(Borders::NONE)
            .title(Span::styled(format!(" {} ", popup_title), Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD)))
            .padding(Padding::horizontal(2));
        
        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let items = app.get_filtered_items();
        let list_items: Vec<ListItem> = if items.is_empty() {
            vec![ListItem::new("   No matches found").style(Style::default().fg(Color::Indexed(235)))]
        } else {
            items.iter().enumerate().map(|(i, entity)| {
                let is_selected = i == app.selected_index;
                let content = if app.popup_mode == PopupMode::Commands {
                    format!(" {:<20}   {} ", entity.id, entity.name)
                } else if is_selected {
                    format!(" ❯ {}  ", entity.name)
                } else {
                    format!("   {}  ", entity.name)
                };
                
                let style = if is_selected {
                    Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Indexed(238))
                };
                ListItem::new(content).style(style)
            }).collect()
        };

        f.render_stateful_widget(
            List::new(list_items).highlight_style(Style::default().bg(Color::Indexed(233))),
            inner_area,
            &mut app.popup_state
        );
    }
}
