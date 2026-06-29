#![allow(dead_code)]
#![allow(dead_code)]
#![allow(dead_code)]
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Main application state for TUI rendering
pub struct AppState {
    pub query: String,
    pub response: String,
    pub context: Vec<String>,
    pub status: String,
    pub input_mode: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            query: String::new(),
            response: String::new(),
            context: Vec::new(),
            status: String::from("Ready"),
            input_mode: true,
        }
    }
}

/// Render the terminal UI
pub fn render(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(frame.size());

    // Title bar
    let title = Block::default()
        .borders(Borders::ALL)
        .title(" 🧠 BNN Code Agent ")
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(title, chunks[0]);

    // Response area
    let response_text = if state.response.is_empty() {
        Text::from(Line::from(Span::styled(
            "Ask a question about your codebase. Type your query below.",
            Style::default().fg(Color::Gray),
        )))
    } else {
        Text::from(Line::from(Span::styled(
            &state.response,
            Style::default().fg(Color::White),
        )))
    };

    let response_block = Block::default()
        .borders(Borders::ALL)
        .title(" Response ")
        .style(Style::default().fg(Color::Green));
    let response_para = Paragraph::new(response_text)
        .block(response_block)
        .wrap(Wrap { trim: false });
    frame.render_widget(response_para, chunks[1]);

    // Context area
    let context_items: Vec<ListItem> = state
        .context
        .iter()
        .map(|c| {
            ListItem::new(Line::from(Span::styled(
                format!("  📄 {}", c),
                Style::default().fg(Color::Yellow),
            )))
        })
        .collect();

    let context_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Context ({} files)", state.context.len()))
        .style(Style::default().fg(Color::Magenta));
    let context_list = List::new(context_items).block(context_block);
    frame.render_widget(context_list, chunks[2]);

    // Status bar
    let status_text = Line::from(vec![
        Span::styled(
            &state.status,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            "Press Ctrl+C to quit, type to query",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    let status_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Blue));
    let status_para = Paragraph::new(Text::from(status_text)).block(status_block);
    frame.render_widget(status_para, chunks[3]);
}
