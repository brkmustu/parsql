//! Migration detail view component

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct MigrationDetailView {
    // TODO: Add actual migration content
}

impl MigrationDetailView {
    pub fn new() -> Self {
        Self {}
    }
    
    // Migration detail view doesn't need to handle keys directly anymore
    // Keys are handled in App::handle_view_key
    
    pub fn render(&mut self, f: &mut Frame, area: Rect, version: i64) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(10),    // Content
                Constraint::Length(4),  // Actions
            ])
            .split(area);
        
        // Header
        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("Migration "),
                Span::styled(
                    format!("v{}", version),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]),
        ])
        .block(Block::default().borders(Borders::ALL).title("Details"));
        f.render_widget(header, chunks[0]);
        
        // Content will be loaded from actual migration files when connected to database
        let content = vec![
            Line::from(vec![
                Span::raw("No migration content available"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Connect to a database to view migration details"),
                Span::styled(" (/connect)", Style::default().fg(Color::DarkGray)),
            ]),
        ];
        
        let content_widget = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title("Migration SQL"))
            .wrap(Wrap { trim: true });
        f.render_widget(content_widget, chunks[1]);
        
        // Actions
        let actions = vec![
            Line::from(vec![
                Span::styled("r ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw("Run this migration  "),
                Span::styled("b ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw("Rollback to before this  "),
                Span::styled("ESC/q ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw("Back to list"),
            ]),
        ];
        
        let actions_widget = Paragraph::new(actions)
            .block(Block::default().borders(Borders::ALL).title("Actions"));
        f.render_widget(actions_widget, chunks[2]);
    }
}