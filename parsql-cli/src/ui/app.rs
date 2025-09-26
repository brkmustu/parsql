//! Main application state and UI rendering

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::config::Config;
use super::command_input::CommandInput;
use super::components::{render_header, render_status_bar};
use super::migration_list::MigrationListView;
use super::migration_detail::MigrationDetailView;
use super::help::HelpView;
use super::output_stream::OutputStreamWidget;
use super::theme::ClaudeTheme;
use super::database::DatabaseInfo;
use super::migration_creator::MigrationCreator;
use super::migration_loader::MigrationLoader;
use super::migration_executor::MigrationExecutor;
use super::migration_viewer::{MigrationViewer, MigrationFileType};
use super::migration_content_view::MigrationContentView;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    CommandInput,
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    MigrationList,
    MigrationDetail { version: i64 },
    DatabaseConfig,
    Logs,
}

pub struct App {
    pub mode: AppMode,
    pub view: View,
    pub command_input: CommandInput,
    pub migration_list: MigrationListView,
    pub migration_detail: MigrationDetailView,
    pub help_view: HelpView,
    pub output_stream: OutputStreamWidget,
    pub migration_content_view: MigrationContentView,
    pub database_url: Option<String>,
    pub config: Config,
    pub verbose: bool,
    pub messages: Vec<(String, MessageType)>,
    pub should_quit: bool,
}

#[derive(Debug, Clone)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}

impl App {
    pub fn new(database_url: Option<String>, config: Config, verbose: bool) -> Self {
        let mut app = Self {
            mode: AppMode::Normal,
            view: View::MigrationList,
            command_input: CommandInput::new(),
            migration_list: MigrationListView::new(),
            migration_detail: MigrationDetailView::new(),
            help_view: HelpView::new(),
            output_stream: OutputStreamWidget::new(1000),
            migration_content_view: MigrationContentView::new(),
            database_url,
            config,
            verbose,
            messages: Vec::new(),
            should_quit: false,
        };
        
        // Load initial data
        app.refresh_data();
        
        app
    }
    
    pub fn refresh_data(&mut self) {
        // Load migrations based on database connection
        if let Some(ref db_url) = self.database_url {
            self.output_stream.add_info("Refreshing migration data...".to_string());
            
            // Load migrations from directory and database
            let migrations_dir = std::path::PathBuf::from(&self.config.migrations.directory);
            let loader = MigrationLoader::new(migrations_dir, self.config.migrations.to_parsql_migrations_config());
            
            // Load migration files
            match loader.load_sql_migrations() {
                Ok(sql_migrations) => {
                    self.output_stream.add_info(format!("Found {} migration files", sql_migrations.len()));
                    
                    // Get migration status (blocking for now)
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();
                    match rt.block_on(loader.get_migration_status(db_url)) {
                        Ok(statuses) => {
                            // Update migration list view
                            self.migration_list.set_migrations(statuses);
                            
                            let applied_count = self.migration_list.migrations.iter()
                                .filter(|m| m.applied)
                                .count();
                            let pending_count = self.migration_list.migrations.len() - applied_count;
                            
                            self.output_stream.add_success(format!(
                                "Loaded {} migrations ({} applied, {} pending)",
                                self.migration_list.migrations.len(),
                                applied_count,
                                pending_count
                            ));
                        }
                        Err(e) => {
                            self.output_stream.add_error(format!("Failed to get migration status: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.output_stream.add_error(format!("Failed to load migrations: {}", e));
                }
            }
        } else {
            self.output_stream.add_warning("No database connection. Use /connect to set database URL".to_string());
            self.add_message("No database connection. Use /connect to set database URL".to_string(), MessageType::Warning);
        }
    }
    
    pub fn add_message(&mut self, message: String, msg_type: MessageType) {
        self.messages.push((message, msg_type));
        // Keep only last 10 messages
        if self.messages.len() > 10 {
            self.messages.remove(0);
        }
    }
    
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        match self.mode {
            AppMode::Normal => self.handle_normal_mode_key(key),
            AppMode::CommandInput => self.handle_command_input_key(key),
            AppMode::Help => self.handle_help_mode_key(key),
        }
    }
    
    fn handle_normal_mode_key(&mut self, key: KeyEvent) -> Result<bool> {
        // Check if migration content view is visible
        if self.migration_content_view.is_visible() {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.migration_content_view.hide();
                    Ok(false)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.migration_content_view.scroll_up();
                    Ok(false)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.migration_content_view.scroll_down(20); // Approximate viewport height
                    Ok(false)
                }
                KeyCode::PageUp => {
                    self.migration_content_view.scroll_page_up(20);
                    Ok(false)
                }
                KeyCode::PageDown => {
                    self.migration_content_view.scroll_page_down(20);
                    Ok(false)
                }
                _ => Ok(false)
            }
        } else {
            match key.code {
                KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                    Ok(true)
                }
                KeyCode::Char('/') => {
                    self.mode = AppMode::CommandInput;
                    self.command_input.clear();
                    // Initialize with '/' character
                    self.command_input.handle_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));
                    Ok(false)
                }
                KeyCode::Char('?') => {
                    self.mode = AppMode::Help;
                    Ok(false)
                }
                KeyCode::Tab => {
                    // Switch between views
                    self.view = match self.view {
                        View::MigrationList => View::Logs,
                        View::MigrationDetail { .. } => View::MigrationList,
                        View::DatabaseConfig => View::MigrationList,
                        View::Logs => View::MigrationList,
                    };
                    Ok(false)
                }
                _ => {
                    // Pass key to current view
                    self.handle_view_key(key)
                }
            }
        }
    }
    
    fn handle_command_input_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.command_input.clear();
                Ok(false)
            }
            KeyCode::Enter => {
                let command = self.command_input.get_command();
                self.mode = AppMode::Normal;
                self.execute_command(&command)?;
                self.command_input.clear();
                Ok(false)
            }
            KeyCode::Tab => {
                self.command_input.complete_suggestion();
                Ok(false)
            }
            _ => {
                self.command_input.handle_key(key);
                Ok(false)
            }
        }
    }
    
    fn handle_help_mode_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = AppMode::Normal;
                Ok(false)
            }
            _ => {
                self.help_view.handle_key(key);
                Ok(false)
            }
        }
    }
    
    fn execute_command(&mut self, command: &str) -> Result<()> {
        // Log the command being executed
        self.output_stream.add_command(command.to_string());
        
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }
        
        match parts[0] {
            "/help" | "/h" => {
                self.mode = AppMode::Help;
            }
            "/quit" | "/q" => {
                self.should_quit = true;
            }
            "/connect" => {
                if parts.len() > 1 {
                    let db_url = parts[1..].join(" ");
                    self.output_stream.add_info(format!("Connecting to database: {}", db_url));
                    
                    match DatabaseInfo::parse(&db_url) {
                        Ok(db_info) => {
                            self.output_stream.add_info(format!("Parsed connection: {}", db_info.display_path));
                            
                            // Test the connection
                            match db_info.test_connection() {
                                Ok(_) => {
                                    self.database_url = Some(db_info.connection_string.clone());
                                    self.output_stream.add_success(format!("Successfully connected to: {}", db_info.display_path));
                                    self.add_message(format!("Connected to: {}", db_info.display_path), MessageType::Success);
                                    
                                    // For SQLite, show the actual file path
                                    if let super::database::DatabaseType::SQLite = db_info.db_type {
                                        if let Some(path) = db_info.connection_string.strip_prefix("sqlite:") {
                                            if path != ":memory:" {
                                                self.output_stream.add_info(format!("Database file: {}", path));
                                            }
                                        }
                                    }
                                    
                                    self.refresh_data();
                                }
                                Err(e) => {
                                    self.output_stream.add_error(format!("Connection failed: {}", e));
                                    self.add_message(format!("Connection failed: {}", e), MessageType::Error);
                                }
                            }
                        }
                        Err(e) => {
                            self.output_stream.add_error(format!("Invalid database URL: {}", e));
                            self.add_message(format!("Invalid database URL: {}", e), MessageType::Error);
                        }
                    }
                } else {
                    self.output_stream.add_error("Usage: /connect <database_url>".to_string());
                    self.add_message("Usage: /connect <database_url>".to_string(), MessageType::Error);
                }
            }
            "/create" => {
                if parts.len() > 1 {
                    let name = parts[1..].join("_");
                    let migration_type = "sql"; // Default to SQL migrations
                    
                    self.output_stream.add_info(format!("Creating {} migration: {}", migration_type.to_uppercase(), name));
                    self.output_stream.add_progress("Generating migration files...".to_string());
                    
                    // Get migrations directory from config or use default
                    let migrations_dir = std::path::PathBuf::from(&self.config.migrations.directory);
                    let creator = MigrationCreator::new(migrations_dir.clone());
                    
                    match creator.create_migration(&name, migration_type) {
                        Ok(files) => {
                            self.output_stream.add_success(format!("Created migration: {} (v{})", files.name, files.version));
                            self.output_stream.add_info(format!("  Up file: {}", files.up_file));
                            if let Some(down_file) = files.down_file {
                                self.output_stream.add_info(format!("  Down file: {}", down_file));
                            }
                            self.output_stream.add_info(format!("Edit the migration files in: {}", migrations_dir.display()));
                            self.add_message(format!("Created migration: {}", name), MessageType::Success);
                            
                            // Refresh migration list
                            self.refresh_data();
                        }
                        Err(e) => {
                            self.output_stream.add_error(format!("Failed to create migration: {}", e));
                            self.add_message(format!("Failed to create migration: {}", e), MessageType::Error);
                        }
                    }
                } else {
                    self.output_stream.add_error("Usage: /create <migration_name>".to_string());
                    self.add_message("Usage: /create <migration_name>".to_string(), MessageType::Error);
                }
            }
            "/run" => {
                if self.database_url.is_none() {
                    self.output_stream.add_error("No database connection. Use /connect first".to_string());
                    self.add_message("No database connection".to_string(), MessageType::Error);
                    return Ok(());
                }
                
                let db_url = self.database_url.as_ref().unwrap();
                self.output_stream.add_info("Checking for pending migrations...".to_string());
                
                // Load migrations
                let migrations_dir = std::path::PathBuf::from(&self.config.migrations.directory);
                let loader = MigrationLoader::new(migrations_dir.clone(), self.config.migrations.to_parsql_migrations_config());
                
                match loader.load_sql_migrations() {
                    Ok(sql_migrations) => {
                        // Filter pending migrations
                        let pending_count = self.migration_list.get_pending_count();
                        if pending_count == 0 {
                            self.output_stream.add_info("No pending migrations to run".to_string());
                            return Ok(());
                        }
                        
                        self.output_stream.add_progress(format!("Running {} pending migrations...", pending_count));
                        
                        // Execute migrations
                        let executor = MigrationExecutor::new(self.config.migrations.to_parsql_migrations_config());
                        
                        if db_url.starts_with("sqlite:") {
                            let db_path = db_url.strip_prefix("sqlite:").unwrap();
                            match executor.run_sqlite_migrations(db_path, sql_migrations, &mut self.output_stream) {
                                Ok(count) => {
                                    self.output_stream.add_success(format!("Successfully ran {} migrations", count));
                                    self.add_message(format!("Ran {} migrations", count), MessageType::Success);
                                    self.refresh_data(); // Refresh to show updated status
                                }
                                Err(e) => {
                                    self.output_stream.add_error(format!("Migration failed: {}", e));
                                    self.add_message(format!("Migration failed: {}", e), MessageType::Error);
                                }
                            }
                        } else {
                            self.output_stream.add_error("PostgreSQL support not yet implemented".to_string());
                        }
                    }
                    Err(e) => {
                        self.output_stream.add_error(format!("Failed to load migrations: {}", e));
                    }
                }
            }
            "/rollback" => {
                if self.database_url.is_none() {
                    self.output_stream.add_error("No database connection. Use /connect first".to_string());
                    self.add_message("No database connection".to_string(), MessageType::Error);
                    return Ok(());
                }
                
                if parts.len() > 1 {
                    if let Ok(target_version) = parts[1].parse::<i64>() {
                        let db_url = self.database_url.as_ref().unwrap();
                        self.output_stream.add_info(format!("Rolling back to version: {}", target_version));
                        
                        // Load migrations
                        let migrations_dir = std::path::PathBuf::from(&self.config.migrations.directory);
                        let loader = MigrationLoader::new(migrations_dir.clone(), self.config.migrations.to_parsql_migrations_config());
                        
                        match loader.load_sql_migrations() {
                            Ok(sql_migrations) => {
                                let executor = MigrationExecutor::new(self.config.migrations.to_parsql_migrations_config());
                                
                                if db_url.starts_with("sqlite:") {
                                    let db_path = db_url.strip_prefix("sqlite:").unwrap();
                                    match executor.rollback_sqlite(db_path, target_version, sql_migrations, &mut self.output_stream) {
                                        Ok(count) => {
                                            self.output_stream.add_success(format!("Successfully rolled back {} migrations", count));
                                            self.add_message(format!("Rolled back {} migrations", count), MessageType::Success);
                                            self.refresh_data(); // Refresh to show updated status
                                        }
                                        Err(e) => {
                                            self.output_stream.add_error(format!("Rollback failed: {}", e));
                                            self.add_message(format!("Rollback failed: {}", e), MessageType::Error);
                                        }
                                    }
                                } else {
                                    self.output_stream.add_error("PostgreSQL support not yet implemented".to_string());
                                }
                            }
                            Err(e) => {
                                self.output_stream.add_error(format!("Failed to load migrations: {}", e));
                            }
                        }
                    } else {
                        self.output_stream.add_error("Invalid version number".to_string());
                        self.add_message("Invalid version number".to_string(), MessageType::Error);
                    }
                } else {
                    self.output_stream.add_error("Usage: /rollback <version>".to_string());
                    self.add_message("Usage: /rollback <version>".to_string(), MessageType::Error);
                }
            }
            "/status" => {
                self.view = View::MigrationList;
                self.refresh_data();
            }
            "/logs" => {
                self.view = View::Logs;
            }
            "/view" => {
                if parts.len() > 1 {
                    if let Ok(version) = parts[1].parse::<i64>() {
                        let file_type = if parts.len() > 2 && parts[2] == "down" {
                            MigrationFileType::Down
                        } else {
                            MigrationFileType::Up
                        };
                        
                        let migrations_dir = std::path::PathBuf::from(&self.config.migrations.directory);
                        let viewer = MigrationViewer::new(migrations_dir);
                        
                        match viewer.view_migration(version, file_type, &mut self.output_stream) {
                            Ok(content) => {
                                let title = format!("Migration {} ({})", version, if matches!(file_type, MigrationFileType::Up) { "up" } else { "down" });
                                self.migration_content_view.show_content(title, content);
                            }
                            Err(e) => {
                                self.output_stream.add_error(format!("Failed to view migration: {}", e));
                                self.add_message(format!("Failed to view migration: {}", e), MessageType::Error);
                            }
                        }
                    } else {
                        self.output_stream.add_error("Invalid version number".to_string());
                    }
                } else {
                    self.output_stream.add_error("Usage: /view <version> [up|down]".to_string());
                    self.add_message("Usage: /view <version> [up|down]".to_string(), MessageType::Error);
                }
            }
            "/edit" => {
                if parts.len() > 1 {
                    if let Ok(version) = parts[1].parse::<i64>() {
                        let file_type = if parts.len() > 2 && parts[2] == "down" {
                            MigrationFileType::Down
                        } else {
                            MigrationFileType::Up
                        };
                        
                        let migrations_dir = std::path::PathBuf::from(&self.config.migrations.directory);
                        let viewer = MigrationViewer::new(migrations_dir);
                        
                        self.output_stream.add_info("Launching editor...".to_string());
                        
                        // Note: This will block the TUI until editor closes
                        // In a real implementation, you might want to save state and restore after
                        match viewer.edit_migration(version, file_type, &mut self.output_stream) {
                            Ok(_) => {
                                self.output_stream.add_success("Editor closed".to_string());
                                self.add_message("Migration edited successfully".to_string(), MessageType::Success);
                            }
                            Err(e) => {
                                self.output_stream.add_error(format!("Failed to edit migration: {}", e));
                                self.add_message(format!("Failed to edit migration: {}", e), MessageType::Error);
                            }
                        }
                    } else {
                        self.output_stream.add_error("Invalid version number".to_string());
                    }
                } else {
                    self.output_stream.add_error("Usage: /edit <version> [up|down]".to_string());
                    self.add_message("Usage: /edit <version> [up|down]".to_string(), MessageType::Error);
                }
            }
            _ => {
                self.output_stream.add_error(format!("Unknown command: {}", parts[0]));
                self.add_message(format!("Unknown command: {}", parts[0]), MessageType::Error);
            }
        }
        
        Ok(())
    }
    
    fn handle_view_key(&mut self, key: KeyEvent) -> Result<bool> {
        match &self.view {
            View::MigrationList => {
                // Handle migration list keys
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.migration_list.previous();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.migration_list.next();
                    }
                    KeyCode::Enter => {
                        if let Some(version) = self.migration_list.get_selected_version() {
                            self.view = View::MigrationDetail { version };
                        }
                    }
                    KeyCode::Char('r') => {
                        self.add_message("Refreshing migration list...".to_string(), MessageType::Info);
                        self.refresh_data();
                    }
                    KeyCode::Char('a') => {
                        let pending_count = self.migration_list.get_pending_count();
                        if pending_count > 0 {
                            self.add_message(
                                format!("Running {} pending migrations...", pending_count),
                                MessageType::Info,
                            );
                            // TODO: Actually run migrations
                        } else {
                            self.add_message(
                                "No pending migrations to run".to_string(),
                                MessageType::Warning,
                            );
                        }
                    }
                    _ => {}
                }
            }
            View::MigrationDetail { .. } => {
                // Handle migration detail keys
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        self.view = View::MigrationList;
                    }
                    KeyCode::Char('r') => {
                        self.add_message("Running this migration...".to_string(), MessageType::Info);
                        // TODO: Actually run the specific migration
                    }
                    KeyCode::Char('b') => {
                        self.add_message("Rolling back to before this migration...".to_string(), MessageType::Info);
                        // TODO: Actually rollback
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(false)
    }
    
    pub fn tick(&mut self) {
        // Update any time-based state
    }
    
    pub fn draw(&mut self, f: &mut Frame) {
        // Set background color
        f.render_widget(
            Block::default().style(Style::default().bg(ClaudeTheme::BG_PRIMARY)),
            f.area(),
        );
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(10),    // Main content area
                Constraint::Length(3),  // Status bar / Command input
            ])
            .split(f.area());
        
        // Render header
        render_header(f, chunks[0], &self.database_url);
        
        // Split main content area into two columns
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),  // Left panel (migrations/config)
                Constraint::Percentage(50),  // Right panel (output stream)
            ])
            .split(chunks[1]);
        
        // Render left panel
        match &self.view {
            View::MigrationList => self.migration_list.render(f, main_chunks[0]),
            View::MigrationDetail { version } => self.migration_detail.render(f, main_chunks[0], *version),
            View::DatabaseConfig => self.render_database_config(f, main_chunks[0]),
            View::Logs => {
                // In logs view, use full width for output stream
                self.output_stream.render(f, chunks[1], "Output Stream");
            }
        }
        
        // Render output stream in right panel (except in logs view)
        if !matches!(self.view, View::Logs) {
            self.output_stream.render(f, main_chunks[1], "Output");
        }
        
        // Render status bar or command input
        match self.mode {
            AppMode::CommandInput => {
                self.command_input.render(f, chunks[2]);
            }
            _ => {
                render_status_bar(f, chunks[2], &self.view, &self.mode);
            }
        }
        
        // Render help overlay if in help mode
        if self.mode == AppMode::Help {
            self.help_view.render(f, f.area());
        }
        
        // Render migration content view if visible
        if self.migration_content_view.is_visible() {
            let area = centered_rect(80, 80, f.area());
            self.migration_content_view.render(f, area);
        }
    }
    
    fn render_messages(&self, _f: &mut Frame, _area: Rect) {
        // Removed - using output stream instead
    }
    
    fn render_database_config(&self, f: &mut Frame, area: Rect) {
        let config_text = vec![
            Line::from(vec![
                Span::raw("Database URL: "),
                Span::styled(
                    self.database_url.as_deref().unwrap_or("Not configured"),
                    Style::default().fg(Color::Yellow)
                ),
            ]),
            Line::from(""),
            Line::from("Migration Settings:"),
            Line::from(format!("  Directory: {}", self.config.migrations.directory)),
            Line::from(format!("  Table Name: {}", self.config.migrations.table_name)),
            Line::from(format!("  Transaction per migration: {}", self.config.migrations.transaction_per_migration)),
            Line::from(format!("  Verify checksums: {}", self.config.migrations.verify_checksums)),
        ];
        
        let paragraph = Paragraph::new(config_text)
            .block(Block::default().borders(Borders::ALL).title("Database Configuration"))
            .wrap(Wrap { trim: true });
        
        f.render_widget(paragraph, area);
    }
    
    fn render_logs(&self, f: &mut Frame, area: Rect) {
        let logs_text = self.messages
            .iter()
            .map(|(msg, msg_type)| {
                let prefix = match msg_type {
                    MessageType::Info => "[INFO] ",
                    MessageType::Success => "[SUCCESS] ",
                    MessageType::Warning => "[WARN] ",
                    MessageType::Error => "[ERROR] ",
                };
                Line::from(format!("{}{}", prefix, msg))
            })
            .collect::<Vec<_>>();
        
        let paragraph = Paragraph::new(logs_text)
            .block(Block::default().borders(Borders::ALL).title("Logs"))
            .wrap(Wrap { trim: true });
        
        f.render_widget(paragraph, area);
    }
}

/// Helper function to create a centered rect
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