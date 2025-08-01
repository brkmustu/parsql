//! Migration list view component

use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};
use crate::ui::theme::ClaudeTheme;
use crate::ui::migration_loader::MigrationStatus;

#[derive(Debug, Clone)]
pub struct MigrationInfo {
    pub version: i64,
    pub name: String,
    pub applied: bool,
    pub applied_at: Option<String>,
    pub checksum: Option<String>,
}

pub struct MigrationListView {
    pub migrations: Vec<MigrationInfo>,
    state: TableState,
}

impl MigrationListView {
    pub fn new() -> Self {
        // Start with empty migrations list
        // Will be populated when connected to a database
        let migrations = vec![];
        let state = TableState::default();
        
        Self { migrations, state }
    }
    
    pub fn set_migrations(&mut self, statuses: Vec<MigrationStatus>) {
        self.migrations = statuses.into_iter().map(|s| MigrationInfo {
            version: s.version,
            name: s.name,
            applied: s.applied,
            applied_at: s.applied_at,
            checksum: None, // TODO: Load checksums
        }).collect();
        
        // Reset selection if needed
        if self.state.selected().map(|i| i >= self.migrations.len()).unwrap_or(false) {
            self.state.select(None);
        }
    }
    
    pub fn get_selected_version(&self) -> Option<i64> {
        self.state.selected().and_then(|i| self.migrations.get(i).map(|m| m.version))
    }
    
    pub fn get_pending_count(&self) -> usize {
        self.migrations.iter().filter(|m| !m.applied).count()
    }
    
    pub fn next(&mut self) {
        if self.migrations.is_empty() {
            return;
        }
        
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.migrations.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    pub fn previous(&mut self) {
        if self.migrations.is_empty() {
            return;
        }
        
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.migrations.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let header_cells = ["Version", "Name", "Status", "Applied At", "Checksum"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(ClaudeTheme::TEXT_PRIMARY).add_modifier(Modifier::BOLD)));
        
        let header = Row::new(header_cells)
            .style(Style::default())
            .height(1);
        
        let rows = self.migrations.iter().map(|migration| {
            let status = if migration.applied {
                Cell::from("✓ Applied").style(Style::default().fg(ClaudeTheme::ACCENT_SUCCESS))
            } else {
                Cell::from("⏳ Pending").style(Style::default().fg(ClaudeTheme::ACCENT_WARNING))
            };
            
            Row::new(vec![
                Cell::from(migration.version.to_string()),
                Cell::from(migration.name.clone()),
                status,
                Cell::from(migration.applied_at.as_deref().unwrap_or("-")),
                Cell::from(migration.checksum.as_deref().unwrap_or("-")),
            ])
            .height(1)
        });
        
        let widths = [
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Length(12),
            Constraint::Length(20),
            Constraint::Length(10),
        ];
        
        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ClaudeTheme::BORDER_PRIMARY))
                .title("Migrations")
                .title_style(Style::default().fg(ClaudeTheme::TEXT_PRIMARY).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(ClaudeTheme::BG_SECONDARY)))
            .row_highlight_style(Style::default().bg(ClaudeTheme::BG_TERTIARY).add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ ");
        
        f.render_stateful_widget(table, area, &mut self.state);
    }
}