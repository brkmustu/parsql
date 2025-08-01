//! Help view component

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub struct HelpView {
    scroll: u16,
}

impl HelpView {
    pub fn new() -> Self {
        Self { scroll: 0 }
    }
    
    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll += 1;
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.scroll += 10;
            }
            _ => {}
        }
    }
    
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Create centered popup
        let popup_area = centered_rect(80, 90, area);
        
        // Clear the area and draw popup background
        f.render_widget(Clear, popup_area);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(10),    // Content
            ])
            .split(popup_area);
        
        // Title
        let title = Paragraph::new("Parsql CLI - Help")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT));
        f.render_widget(title, chunks[0]);
        
        // Help content
        let help_text = vec![
            Line::from(vec![Span::styled("Navigation", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from("  ↑/k         Move up"),
            Line::from("  ↓/j         Move down"),
            Line::from("  Enter       Select/Open"),
            Line::from("  Tab         Switch view"),
            Line::from("  ESC/q       Go back / Close"),
            Line::from("  Ctrl+q      Quit application"),
            Line::from(""),
            Line::from(vec![Span::styled("Commands (press / to enter command mode)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from("  /help       Show this help"),
            Line::from("  /quit       Exit the application"),
            Line::from("  /connect    Connect to database"),
            Line::from("  /create     Create new migration"),
            Line::from("  /run        Run pending migrations"),
            Line::from("  /rollback   Rollback to version"),
            Line::from("  /status     Show migration status"),
            Line::from("  /validate   Validate migrations"),
            Line::from("  /list       List migrations"),
            Line::from("  /logs       Show application logs"),
            Line::from("  /config     Show configuration"),
            Line::from("  /refresh    Refresh migration data"),
            Line::from(""),
            Line::from(vec![Span::styled("Migration List Shortcuts", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from("  r           Refresh list"),
            Line::from("  a           Run all pending migrations"),
            Line::from(""),
            Line::from(vec![Span::styled("Migration Detail Shortcuts", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from("  r           Run this migration"),
            Line::from("  b           Rollback to before this migration"),
            Line::from(""),
            Line::from(vec![Span::styled("Command Input Mode", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from("  Tab         Complete suggestion"),
            Line::from("  ↑/↓         Navigate suggestions"),
            Line::from("  Enter       Execute command"),
            Line::from("  ESC         Cancel command"),
            Line::from(""),
            Line::from(vec![Span::styled("Tips", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from("• Type '/' to quickly access commands"),
            Line::from("• Commands support auto-completion"),
            Line::from("• Use Tab to switch between different views"),
            Line::from("• All data is automatically refreshed after operations"),
        ];
        
        let help_content = Paragraph::new(help_text)
            .block(Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .title("Press ESC or q to close"))
            .wrap(Wrap { trim: true })
            .scroll((self.scroll, 0));
        
        f.render_widget(help_content, chunks[1]);
    }
}

/// Helper function to create centered rect
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