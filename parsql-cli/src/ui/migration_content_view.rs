//! Migration content display widget

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use super::theme::ModernTheme;

pub struct MigrationContentView {
    content: Vec<String>,
    scroll_offset: u16,
    scroll_state: ScrollbarState,
    is_visible: bool,
    title: String,
}

impl MigrationContentView {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            scroll_offset: 0,
            scroll_state: ScrollbarState::new(0),
            is_visible: false,
            title: String::new(),
        }
    }
    
    pub fn show_content(&mut self, title: String, content: String) {
        self.title = title;
        self.content = content.lines().map(|l| l.to_string()).collect();
        self.scroll_offset = 0;
        self.scroll_state = ScrollbarState::new(self.content.len());
        self.is_visible = true;
    }
    
    pub fn hide(&mut self) {
        self.is_visible = false;
    }
    
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
    
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
            self.scroll_state = self.scroll_state.position(self.scroll_offset as usize);
        }
    }
    
    pub fn scroll_down(&mut self, viewport_height: u16) {
        let max_scroll = self.content.len().saturating_sub(viewport_height as usize) as u16;
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
            self.scroll_state = self.scroll_state.position(self.scroll_offset as usize);
        }
    }
    
    pub fn scroll_page_up(&mut self, viewport_height: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(viewport_height);
        self.scroll_state = self.scroll_state.position(self.scroll_offset as usize);
    }
    
    pub fn scroll_page_down(&mut self, viewport_height: u16) {
        let max_scroll = self.content.len().saturating_sub(viewport_height as usize) as u16;
        self.scroll_offset = (self.scroll_offset + viewport_height).min(max_scroll);
        self.scroll_state = self.scroll_state.position(self.scroll_offset as usize);
    }
    
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if !self.is_visible {
            return;
        }
        
        // Create styled lines with syntax highlighting
        let mut lines = Vec::new();
        
        // Calculate visible range
        let viewport_height = area.height.saturating_sub(2) as usize; // Account for borders
        let start = self.scroll_offset as usize;
        let end = (start + viewport_height).min(self.content.len());
        
        for (i, line) in self.content[start..end].iter().enumerate() {
            let line_number = start + i + 1;
            let styled_line = self.highlight_sql_line(line, line_number);
            lines.push(styled_line);
        }
        
        // Create the main content block
        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ModernTheme::BORDER))
            .title_style(Style::default().fg(ModernTheme::TEXT_PRIMARY).add_modifier(Modifier::BOLD));
        
        let paragraph = Paragraph::new(lines)
            .block(block)
            .style(Style::default().fg(ModernTheme::TEXT_PRIMARY).bg(ModernTheme::BG_SECONDARY));
        
        f.render_widget(paragraph, area);
        
        // Render scrollbar if content is longer than viewport
        if self.content.len() > viewport_height {
            let scrollbar_area = Rect {
                x: area.x + area.width - 1,
                y: area.y + 1,
                width: 1,
                height: area.height - 2,
            };
            
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"));
            
            f.render_stateful_widget(scrollbar, scrollbar_area, &mut self.scroll_state);
        }
        
        // Render help text at bottom
        let help_text = " ↑/↓: scroll | PgUp/PgDn: page | q/Esc: close ";
        let help_span = Span::styled(help_text, Style::default().fg(ModernTheme::TEXT_MUTED));
        let help_x = area.x + area.width.saturating_sub(help_text.len() as u16 + 1);
        f.render_widget(help_span, Rect { x: help_x, y: area.y + area.height - 1, width: help_text.len() as u16, height: 1 });
    }
    
    fn highlight_sql_line<'a>(&self, line: &'a str, line_number: usize) -> Line<'a> {
        let mut spans = vec![];
        
        // Add line number
        spans.push(Span::styled(
            format!("{:4} │ ", line_number),
            Style::default().fg(ModernTheme::TEXT_MUTED),
        ));
        
        // Simple SQL syntax highlighting
        let trimmed = line.trim_start();
        
        if trimmed.starts_with("--") {
            // SQL comment
            spans.push(Span::styled(line.to_string(), Style::default().fg(ModernTheme::SUCCESS)));
        } else if trimmed.is_empty() {
            // Empty line
            spans.push(Span::raw(line.to_string()));
        } else {
            // Highlight SQL keywords
            let words: Vec<&str> = line.split_whitespace().collect();
            let mut current_pos = 0;
            
            for (i, word) in words.iter().enumerate() {
                // Add spaces before word
                if let Some(pos) = line[current_pos..].find(word) {
                    if pos > 0 {
                        spans.push(Span::raw(&line[current_pos..current_pos + pos]));
                    }
                    current_pos += pos;
                }
                
                let word_upper = word.to_uppercase();
                let style = if is_sql_keyword(&word_upper) {
                    Style::default().fg(ModernTheme::ACCENT_PRIMARY).add_modifier(Modifier::BOLD)
                } else if is_sql_type(&word_upper) {
                    Style::default().fg(ModernTheme::WARNING)
                } else if word.starts_with('\'') || word.starts_with('"') {
                    Style::default().fg(ModernTheme::SUCCESS)
                } else {
                    Style::default().fg(ModernTheme::TEXT_PRIMARY)
                };
                
                spans.push(Span::styled(word.to_string(), style));
                current_pos += word.len();
                
                // Add space after word if not last
                if i < words.len() - 1 && current_pos < line.len() {
                    spans.push(Span::raw(" "));
                    current_pos += 1;
                }
            }
            
            // Add any remaining characters
            if current_pos < line.len() {
                spans.push(Span::raw(&line[current_pos..]));
            }
        }
        
        Line::from(spans)
    }
}

fn is_sql_keyword(word: &str) -> bool {
    matches!(
        word,
        "CREATE" | "TABLE" | "DROP" | "ALTER" | "INSERT" | "UPDATE" | "DELETE" | "SELECT" |
        "FROM" | "WHERE" | "JOIN" | "LEFT" | "RIGHT" | "INNER" | "OUTER" | "ON" |
        "GROUP" | "BY" | "ORDER" | "HAVING" | "LIMIT" | "OFFSET" | "UNION" | "ALL" |
        "AS" | "IN" | "EXISTS" | "BETWEEN" | "LIKE" | "IS" | "NOT" | "NULL" |
        "AND" | "OR" | "CASE" | "WHEN" | "THEN" | "ELSE" | "END" |
        "PRIMARY" | "KEY" | "FOREIGN" | "REFERENCES" | "CONSTRAINT" | "UNIQUE" |
        "INDEX" | "DEFAULT" | "AUTO_INCREMENT" | "AUTOINCREMENT" | "IF" | "BEGIN" | 
        "COMMIT" | "ROLLBACK" | "TRANSACTION"
    )
}

fn is_sql_type(word: &str) -> bool {
    matches!(
        word,
        "INTEGER" | "INT" | "BIGINT" | "SMALLINT" | "TINYINT" |
        "DECIMAL" | "NUMERIC" | "FLOAT" | "DOUBLE" | "REAL" |
        "VARCHAR" | "CHAR" | "TEXT" | "BLOB" | "CLOB" |
        "DATE" | "TIME" | "TIMESTAMP" | "DATETIME" |
        "BOOLEAN" | "BOOL" | "BIT" |
        "SERIAL" | "BIGSERIAL" | "UUID"
    )
}