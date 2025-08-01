//! Utility functions for parsql CLI

use anyhow::Result;
use colored::Colorize;
use std::time::Instant;

/// Parse database URL and determine database type
pub fn parse_database_url(url: &str) -> Result<DatabaseType> {
    if url.starts_with("postgresql://") || url.starts_with("postgres://") {
        Ok(DatabaseType::PostgreSQL)
    } else if url.starts_with("sqlite:") || url.ends_with(".db") || url.ends_with(".sqlite") {
        Ok(DatabaseType::SQLite)
    } else {
        anyhow::bail!("Unsupported database URL format. Use postgresql:// or sqlite:")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DatabaseType {
    PostgreSQL,
    SQLite,
}

impl DatabaseType {
    pub fn name(&self) -> &'static str {
        match self {
            DatabaseType::PostgreSQL => "PostgreSQL",
            DatabaseType::SQLite => "SQLite",
        }
    }
}

/// Format duration in a human-readable way
pub fn format_duration(duration: std::time::Duration) -> String {
    let millis = duration.as_millis();
    if millis < 1000 {
        format!("{}ms", millis)
    } else {
        format!("{:.2}s", duration.as_secs_f64())
    }
}

/// Print a success message
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

/// Print an error message
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}

/// Print a warning message
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message);
}

/// Print an info message
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

/// Progress indicator for long-running operations
pub struct Progress {
    #[allow(dead_code)]
    message: String,
    start: Instant,
}

impl Progress {
    pub fn new(message: &str) -> Self {
        print!("{} {}... ", "⟳".cyan().bold(), message);
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
        
        Self {
            message: message.to_string(),
            start: Instant::now(),
        }
    }
    
    pub fn finish(self) {
        let duration = self.start.elapsed();
        println!("{} ({})", "done".green(), format_duration(duration).dimmed());
    }
    
    pub fn finish_with_message(self, message: &str) {
        let duration = self.start.elapsed();
        println!("{} {} ({})", 
            "done".green(), 
            message, 
            format_duration(duration).dimmed()
        );
    }
}

/// Format a table for display
pub fn format_table(headers: Vec<&str>, rows: Vec<Vec<String>>) -> String {
    use std::cmp::max;
    
    // Calculate column widths
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = max(widths[i], cell.len());
            }
        }
    }
    
    let mut output = String::new();
    
    // Print headers
    for (i, header) in headers.iter().enumerate() {
        if i > 0 {
            output.push_str("  ");
        }
        output.push_str(&format!("{:<width$}", header, width = widths[i]));
    }
    output.push('\n');
    
    // Print separator
    for (i, width) in widths.iter().enumerate() {
        if i > 0 {
            output.push_str("  ");
        }
        output.push_str(&"-".repeat(*width));
    }
    output.push('\n');
    
    // Print rows
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i > 0 {
                output.push_str("  ");
            }
            if i < widths.len() {
                output.push_str(&format!("{:<width$}", cell, width = widths[i]));
            }
        }
        output.push('\n');
    }
    
    output
}

/// Get timestamp for migration files
pub fn get_timestamp() -> String {
    chrono::Local::now().format("%Y%m%d%H%M%S").to_string()
}

/// Colorize a number with a label
pub fn colorize_number(num: usize, label: &str) -> String {
    format!("{} {}", num.to_string().bold(), label)
}