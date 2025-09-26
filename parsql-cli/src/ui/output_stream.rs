//! Output stream widget for displaying command outputs

use std::collections::VecDeque;
use chrono::Local;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use super::theme::ClaudeTheme;

#[derive(Debug, Clone)]
pub struct OutputLine {
    pub timestamp: String,
    pub content: String,
    pub line_type: OutputLineType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputLineType {
    Command,
    Info,
    Success,
    Warning,
    Error,
    Progress,
    Result,
}

pub struct OutputStreamWidget {
    lines: VecDeque<OutputLine>,
    max_lines: usize,
    state: ListState,
    auto_scroll: bool,
}

impl OutputStreamWidget {
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(max_lines),
            max_lines,
            state: ListState::default(),
            auto_scroll: true,
        }
    }
    
    pub fn add_command(&mut self, command: String) {
        self.add_line(OutputLine {
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            content: format!("> {}", command),
            line_type: OutputLineType::Command,
        });
    }
    
    pub fn add_info(&mut self, content: String) {
        self.add_line(OutputLine {
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            content,
            line_type: OutputLineType::Info,
        });
    }
    
    pub fn add_success(&mut self, content: String) {
        self.add_line(OutputLine {
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            content: format!("✓ {}", content),
            line_type: OutputLineType::Success,
        });
    }
    
    pub fn add_error(&mut self, content: String) {
        self.add_line(OutputLine {
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            content: format!("✗ {}", content),
            line_type: OutputLineType::Error,
        });
    }
    
    pub fn add_warning(&mut self, content: String) {
        self.add_line(OutputLine {
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            content: format!("⚠ {}", content),
            line_type: OutputLineType::Warning,
        });
    }
    
    pub fn add_progress(&mut self, content: String) {
        // Update last line if it's also progress
        if let Some(last) = self.lines.back() {
            if last.line_type == OutputLineType::Progress {
                self.lines.pop_back();
            }
        }
        
        self.add_line(OutputLine {
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            content,
            line_type: OutputLineType::Progress,
        });
    }
    
    pub fn add_result(&mut self, content: String) {
        self.add_line(OutputLine {
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            content,
            line_type: OutputLineType::Result,
        });
    }
    
    fn add_line(&mut self, line: OutputLine) {
        self.lines.push_back(line);
        
        // Remove old lines if exceeding max
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
        
        // Auto scroll to bottom
        if self.auto_scroll && !self.lines.is_empty() {
            self.state.select(Some(self.lines.len() - 1));
        }
    }
    
    pub fn clear(&mut self) {
        self.lines.clear();
        self.state.select(None);
    }
    
    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll = !self.auto_scroll;
    }
    
    pub fn render(&mut self, f: &mut Frame, area: Rect, title: &str) {
        let items: Vec<ListItem> = self.lines
            .iter()
            .map(|line| {
                let (style, _prefix_style) = match line.line_type {
                    OutputLineType::Command => (
                        Style::default().fg(ClaudeTheme::ACCENT_PRIMARY).add_modifier(Modifier::BOLD),
                        Style::default().fg(ClaudeTheme::ACCENT_PRIMARY),
                    ),
                    OutputLineType::Info => (
                        Style::default().fg(ClaudeTheme::TEXT_PRIMARY),
                        Style::default().fg(ClaudeTheme::TEXT_DIM),
                    ),
                    OutputLineType::Success => (
                        Style::default().fg(ClaudeTheme::ACCENT_SUCCESS),
                        Style::default().fg(ClaudeTheme::ACCENT_SUCCESS),
                    ),
                    OutputLineType::Warning => (
                        Style::default().fg(ClaudeTheme::ACCENT_WARNING),
                        Style::default().fg(ClaudeTheme::ACCENT_WARNING),
                    ),
                    OutputLineType::Error => (
                        Style::default().fg(ClaudeTheme::ACCENT_ERROR),
                        Style::default().fg(ClaudeTheme::ACCENT_ERROR),
                    ),
                    OutputLineType::Progress => (
                        Style::default().fg(ClaudeTheme::ACCENT_INFO).add_modifier(Modifier::ITALIC),
                        Style::default().fg(ClaudeTheme::ACCENT_INFO),
                    ),
                    OutputLineType::Result => (
                        Style::default().fg(ClaudeTheme::TEXT_PRIMARY),
                        Style::default().fg(ClaudeTheme::TEXT_SECONDARY),
                    ),
                };
                
                let content = vec![
                    Span::styled(
                        format!("[{}] ", line.timestamp),
                        Style::default().fg(ClaudeTheme::TEXT_DIM),
                    ),
                    Span::styled(&line.content, style),
                ];
                
                ListItem::new(Line::from(content))
            })
            .collect();
        
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(ClaudeTheme::BORDER_PRIMARY))
                    .title(title)
                    .title_style(
                        Style::default()
                            .fg(ClaudeTheme::TEXT_PRIMARY)
                            .add_modifier(Modifier::BOLD)
                    )
                    .style(Style::default().bg(ClaudeTheme::BG_SECONDARY))
            )
            .highlight_style(
                Style::default()
                    .bg(ClaudeTheme::BG_TERTIARY)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("> ");
        
        f.render_stateful_widget(list, area, &mut self.state);
    }
}