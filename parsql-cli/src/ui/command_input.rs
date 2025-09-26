//! Command input handling with auto-suggestions (Claude Code style)

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::ui::theme::ClaudeTheme;

#[derive(Debug, Clone)]
pub struct CommandSuggestion {
    pub command: String,
    pub description: String,
    pub usage: String,
}

impl CommandSuggestion {
    fn new(command: &str, description: &str, usage: &str) -> Self {
        Self {
            command: command.to_string(),
            description: description.to_string(),
            usage: usage.to_string(),
        }
    }
}

pub struct CommandInput {
    input: String,
    cursor_position: usize,
    suggestions: Vec<CommandSuggestion>,
    selected_suggestion: usize,
    all_commands: Vec<CommandSuggestion>,
}

impl CommandInput {
    pub fn new() -> Self {
        let all_commands = vec![
            CommandSuggestion::new(
                "/help",
                "Show help and keyboard shortcuts",
                "/help or /h"
            ),
            CommandSuggestion::new(
                "/quit",
                "Exit the application",
                "/quit or /q"
            ),
            CommandSuggestion::new(
                "/connect",
                "Connect to a database",
                "/connect <database_url>"
            ),
            CommandSuggestion::new(
                "/create",
                "Create a new migration",
                "/create <migration_name>"
            ),
            CommandSuggestion::new(
                "/run",
                "Run pending migrations",
                "/run [--dry-run] [--target <version>]"
            ),
            CommandSuggestion::new(
                "/rollback",
                "Rollback migrations to a specific version",
                "/rollback <version> [--dry-run]"
            ),
            CommandSuggestion::new(
                "/status",
                "Show migration status",
                "/status [--detailed]"
            ),
            CommandSuggestion::new(
                "/validate",
                "Validate migrations",
                "/validate [--check-gaps] [--verify-checksums]"
            ),
            CommandSuggestion::new(
                "/view",
                "View migration SQL content",
                "/view <version> [up|down]"
            ),
            CommandSuggestion::new(
                "/edit",
                "Edit migration file in external editor",
                "/edit <version> [up|down]"
            ),
            CommandSuggestion::new(
                "/list",
                "List migrations",
                "/list [--pending] [--applied]"
            ),
            CommandSuggestion::new(
                "/logs",
                "Show application logs",
                "/logs"
            ),
            CommandSuggestion::new(
                "/config",
                "Show database configuration",
                "/config"
            ),
            CommandSuggestion::new(
                "/refresh",
                "Refresh migration data",
                "/refresh"
            ),
        ];
        
        Self {
            input: String::new(),
            cursor_position: 0,
            suggestions: all_commands.clone(),
            selected_suggestion: 0,
            all_commands,
        }
    }
    
    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
        self.update_suggestions();
    }
    
    pub fn get_command(&self) -> String {
        self.input.clone()
    }
    
    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
                self.update_suggestions();
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.input.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                    self.update_suggestions();
                }
            }
            KeyCode::Delete => {
                if self.cursor_position < self.input.len() {
                    self.input.remove(self.cursor_position);
                    self.update_suggestions();
                }
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.input.len() {
                    self.cursor_position += 1;
                }
            }
            KeyCode::Home => {
                self.cursor_position = 0;
            }
            KeyCode::End => {
                self.cursor_position = self.input.len();
            }
            KeyCode::Up => {
                if self.selected_suggestion > 0 {
                    self.selected_suggestion -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_suggestion < self.suggestions.len().saturating_sub(1) {
                    self.selected_suggestion += 1;
                }
            }
            _ => {}
        }
    }
    
    pub fn complete_suggestion(&mut self) {
        if let Some(suggestion) = self.suggestions.get(self.selected_suggestion) {
            self.input = suggestion.command.clone();
            self.cursor_position = self.input.len();
            self.update_suggestions();
        }
    }
    
    fn update_suggestions(&mut self) {
        if self.input.is_empty() {
            self.suggestions = self.all_commands.clone();
        } else if self.input.starts_with('/') {
            // Filter commands based on input
            self.suggestions = self.all_commands
                .iter()
                .filter(|cmd| cmd.command.starts_with(&self.input))
                .cloned()
                .collect();
        } else {
            self.suggestions.clear();
        }
        
        // Reset selection if out of bounds
        if self.selected_suggestion >= self.suggestions.len() {
            self.selected_suggestion = 0;
        }
    }
    
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Use the entire area for input
        let input_area = area;
        
        // Only show suggestions when input starts with '/'
        if self.input.starts_with('/') && !self.suggestions.is_empty() {
            // Create a popup area for suggestions (Claude Code style)
            let popup_height = std::cmp::min(self.suggestions.len() as u16 + 2, 15);
            // Position suggestions above the input area
            let popup_area = Rect {
                x: area.x,
                y: area.y.saturating_sub(popup_height + 1),
                width: area.width,
                height: popup_height,
            };
            
            // Render suggestions
            let suggestions: Vec<ListItem> = self.suggestions
                .iter()
                .enumerate()
                .map(|(i, suggestion)| {
                    let style = if i == self.selected_suggestion {
                        Style::default().bg(ClaudeTheme::SELECTION_BG).fg(ClaudeTheme::TEXT_PRIMARY).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(ClaudeTheme::TEXT_PRIMARY)
                    };
                    
                    let content = vec![
                        Line::from(vec![
                            Span::styled(&suggestion.command, Style::default().fg(ClaudeTheme::ACCENT_PRIMARY).add_modifier(if i == self.selected_suggestion { Modifier::BOLD } else { Modifier::empty() })),
                            Span::styled(" - ", Style::default().fg(ClaudeTheme::TEXT_DIM)),
                            Span::styled(&suggestion.description, style),
                        ]),
                    ];
                    
                    ListItem::new(content)
                })
                .collect();
            
            let suggestions_list = List::new(suggestions)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(ClaudeTheme::BORDER_FOCUSED))
                    .title(" Commands ")
                    .title_style(Style::default().fg(ClaudeTheme::ACCENT_PRIMARY).add_modifier(Modifier::BOLD))
                    .style(Style::default().bg(ClaudeTheme::COMMAND_BG)));
            
            // Clear the popup area first
            f.render_widget(Block::default().style(Style::default().bg(ClaudeTheme::COMMAND_BG)), popup_area);
            f.render_widget(suggestions_list, popup_area);
        }
        
        // Render input field with prominent styling
        let input_display = if self.input.is_empty() {
            "/".to_string()
        } else {
            self.input.clone()
        };
        
        let input_widget = Paragraph::new(input_display.as_str())
            .style(Style::default().fg(ClaudeTheme::TEXT_PRIMARY).bg(ClaudeTheme::COMMAND_BG))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ClaudeTheme::BORDER_FOCUSED).add_modifier(Modifier::BOLD))
                .title(" Command (ESC to cancel) ")
                .title_style(Style::default().fg(ClaudeTheme::ACCENT_PRIMARY).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(ClaudeTheme::COMMAND_BG)));
        
        f.render_widget(input_widget, input_area);
        
        // Set cursor position
        f.set_cursor_position((
            input_area.x + self.cursor_position as u16 + 1 + if self.input.is_empty() { 1 } else { 0 },
            input_area.y + 1,
        ));
    }
}