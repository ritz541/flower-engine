use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::COLOR_AI;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let cursor = "█"; 
    let max_width = area.width.saturating_sub(4) as usize; // Account for padding and prompt
    
    if app.is_typing {
        let prompt_line = Line::from(vec![
            Span::styled("  ✦ ", Style::default().fg(COLOR_AI)),
            Span::styled("Narrating...", Style::default().fg(Color::Indexed(236)).add_modifier(Modifier::ITALIC)),
        ]);
        
        f.render_widget(
            Paragraph::new(prompt_line).block(Block::default().bg(Color::Indexed(233))),
            area
        );
    } else {
        let mut lines: Vec<Line> = Vec::new();
        
        if app.input.is_empty() {
            // Show placeholder text when input is empty
            let placeholder = Line::from(vec![
                Span::styled("  › ", Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD)),
                Span::styled("How do you respond? (/ for commands)", Style::default().fg(Color::Indexed(236))),
            ]);
            lines.push(placeholder);
        } else {
            // Handle actual input with wrapping
            let mut current_line = String::new();
            let mut current_line_spans: Vec<Span> = vec![
                Span::styled("  › ", Style::default().fg(COLOR_AI).add_modifier(Modifier::BOLD))
            ];
            
            // Process each character to handle wrapping
            let input_chars: Vec<char> = app.input.chars().collect();
            
            for ch in input_chars.iter() {
                let char_width = ch.len_utf8(); // Handle multi-byte characters properly
                
                // Check if adding this character would exceed width
                if current_line.len() + char_width > max_width {
                    lines.push(Line::from(current_line_spans.clone()));
                    
                    // Start new line
                    current_line = String::new();
                    current_line_spans = vec![Span::raw("    ")]; // Indent for continuation
                }
                
                current_line.push(*ch);
                current_line_spans.push(Span::styled(ch.to_string(), Style::default().fg(Color::White)));
            }
            
            // Add cursor at the end
            current_line_spans.push(Span::styled(cursor, Style::default().fg(COLOR_AI)));
            
            
            // Add command hint if present
            if !app.command_hint.is_empty() {
                current_line_spans.push(Span::styled(app.command_hint.clone(), Style::default().fg(Color::Indexed(234))));
            }
            
            lines.push(Line::from(current_line_spans));
        }
        
        // Ensure we have exactly the right number of lines for the area
        // Fill with empty lines if needed, or truncate if too many
        while lines.len() < area.height as usize {
            lines.push(Line::from(""));
        }
        // Truncate if we have too many lines
        lines.truncate(area.height as usize);
        
        let text = Text::from(lines);
        
        f.render_widget(
            Paragraph::new(text).block(Block::default().bg(Color::Indexed(233))),
            area
        );
    }
}
