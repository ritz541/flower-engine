use crate::app::{App, PopupMode};
use crate::ui::COLOR_AI;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Padding},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_title = match app.popup_mode {
        PopupMode::World => " WORLD ",
        PopupMode::Character => " CHARACTER ",
        PopupMode::Model => " MODEL ",
        PopupMode::Rules => " RULES ",
        PopupMode::Session => " SESSION ",
        PopupMode::Commands => " COMMANDS ",
        _ => " SELECT ",
    };

    let block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(
            format!(" {} ", popup_title),
            Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD),
        ))
        .padding(Padding::horizontal(2));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let items = app.get_filtered_items();
    let list_items: Vec<ListItem> = if items.is_empty() {
        vec![ListItem::new("   No matches found").style(Style::default().fg(Color::Indexed(235)))]
    } else {
        items
            .iter()
            .enumerate()
            .map(|(i, entity)| {
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
            })
            .collect()
    };

    f.render_stateful_widget(
        List::new(list_items).highlight_style(Style::default().bg(Color::Indexed(233))),
        inner_area,
        &mut app.popup_state,
    );
}
