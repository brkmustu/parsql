//! Reusable UI components

use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::app::{AppMode, View};
use super::theme::ClaudeTheme;

/// Render the application header
pub fn render_header(f: &mut Frame, area: Rect, database_url: &Option<String>) {
    let db_status = if let Some(url) = database_url {
        // Hide password in connection string
        let display_url = if url.contains('@') {
            let parts: Vec<&str> = url.split('@').collect();
            if parts.len() == 2 {
                let protocol_and_creds = parts[0];
                let host_and_rest = parts[1];
                
                if let Some(proto_end) = protocol_and_creds.rfind("://") {
                    let protocol = &protocol_and_creds[..proto_end + 3];
                    let creds = &protocol_and_creds[proto_end + 3..];
                    
                    if creds.contains(':') {
                        let user = creds.split(':').next().unwrap_or("");
                        format!("{}{}:****@{}", protocol, user, host_and_rest)
                    } else {
                        url.clone()
                    }
                } else {
                    url.clone()
                }
            } else {
                url.clone()
            }
        } else {
            url.clone()
        };
        
        vec![
            Span::raw("Connected: "),
            Span::styled(display_url, Style::default().fg(ClaudeTheme::ACCENT_SUCCESS)),
        ]
    } else {
        vec![
            Span::raw("Disconnected "),
            Span::styled("(/connect to set database)", Style::default().fg(ClaudeTheme::TEXT_DIM)),
        ]
    };
    
    let header_text = vec![
        Line::from(vec![
            Span::styled(
                "Parsql CLI",
                Style::default()
                    .fg(ClaudeTheme::ACCENT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - "),
            Span::raw("Interactive Migration Manager"),
        ]),
        Line::from(db_status),
    ];
    
    let header = Paragraph::new(header_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ClaudeTheme::BORDER_PRIMARY))
                .style(Style::default()
                    .bg(ClaudeTheme::BG_SECONDARY)
                    .fg(ClaudeTheme::TEXT_PRIMARY)),
        )
        .alignment(Alignment::Center);
    
    f.render_widget(header, area);
}

/// Render the status bar
pub fn render_status_bar(f: &mut Frame, area: Rect, current_view: &View, mode: &AppMode) {
    let mode_indicator = match mode {
        AppMode::Normal => "NORMAL",
        AppMode::CommandInput => "COMMAND",
        AppMode::Help => "HELP",
    };
    
    let view_indicator = match current_view {
        View::MigrationList => "Migrations",
        View::MigrationDetail { .. } => "Migration Detail",
        View::DatabaseConfig => "Configuration",
        View::Logs => "Logs",
    };
    
    let keybinds = match mode {
        AppMode::Normal => {
            vec![
                ("/ ", "Command"),
                ("? ", "Help"),
                ("Tab ", "Switch View"),
                ("q ", "Quit"),
                ("↑↓ ", "Navigate"),
                ("Enter ", "Select"),
            ]
        }
        AppMode::CommandInput => {
            vec![
                ("ESC ", "Cancel"),
                ("Enter ", "Execute"),
                ("Tab ", "Complete"),
                ("↑↓ ", "Suggestions"),
            ]
        }
        AppMode::Help => {
            vec![
                ("ESC/q ", "Close"),
                ("↑↓ ", "Scroll"),
            ]
        }
    };
    
    let mut spans = vec![
        Span::styled(
            format!(" {} ", mode_indicator),
            Style::default()
                .bg(ClaudeTheme::ACCENT_PRIMARY)
                .fg(ClaudeTheme::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            view_indicator,
            Style::default().fg(ClaudeTheme::ACCENT_WARNING),
        ),
        Span::raw(" | "),
    ];
    
    for (key, desc) in keybinds {
        spans.push(Span::styled(
            key,
            Style::default()
                .fg(ClaudeTheme::ACCENT_INFO)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(desc));
        spans.push(Span::raw(" "));
    }
    
    let status = Paragraph::new(Line::from(spans))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ClaudeTheme::BORDER_PRIMARY))
                .style(Style::default()
                    .bg(ClaudeTheme::BG_SECONDARY)
                    .fg(ClaudeTheme::TEXT_PRIMARY)),
        );
    
    f.render_widget(status, area);
}