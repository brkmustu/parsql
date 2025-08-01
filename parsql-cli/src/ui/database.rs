//! Database connection handling

use std::path::Path;
use anyhow::{Context, Result};

pub enum DatabaseType {
    SQLite,
    PostgreSQL,
}

pub struct DatabaseInfo {
    pub db_type: DatabaseType,
    pub connection_string: String,
    pub display_path: String,
}

impl DatabaseInfo {
    pub fn parse(url: &str) -> Result<Self> {
        if url.starts_with("sqlite:") {
            let path = url.strip_prefix("sqlite:").unwrap_or(url);
            
            // Convert relative path to absolute path based on current directory
            let abs_path = if path == ":memory:" {
                ":memory:".to_string()
            } else {
                let current_dir = std::env::current_dir()
                    .context("Failed to get current directory")?;
                let full_path = if Path::new(path).is_absolute() {
                    Path::new(path).to_path_buf()
                } else {
                    current_dir.join(path)
                };
                
                full_path.to_string_lossy().to_string()
            };
            
            Ok(Self {
                db_type: DatabaseType::SQLite,
                connection_string: format!("sqlite:{}", abs_path),
                display_path: format!("SQLite: {}", abs_path),
            })
        } else if url.starts_with("postgresql://") || url.starts_with("postgres://") {
            Ok(Self {
                db_type: DatabaseType::PostgreSQL,
                connection_string: url.to_string(),
                display_path: Self::hide_password(url),
            })
        } else {
            anyhow::bail!("Unsupported database URL format. Use 'sqlite:path/to/db.db' or 'postgresql://...'")
        }
    }
    
    fn hide_password(url: &str) -> String {
        if url.contains('@') {
            let parts: Vec<&str> = url.split('@').collect();
            if parts.len() == 2 {
                let protocol_and_creds = parts[0];
                let host_and_rest = parts[1];
                
                if let Some(proto_end) = protocol_and_creds.rfind("://") {
                    let protocol = &protocol_and_creds[..proto_end + 3];
                    let creds = &protocol_and_creds[proto_end + 3..];
                    
                    if creds.contains(':') {
                        let user = creds.split(':').next().unwrap_or("");
                        return format!("{}{}:****@{}", protocol, user, host_and_rest);
                    }
                }
            }
        }
        url.to_string()
    }
    
    pub fn test_connection(&self) -> Result<()> {
        match self.db_type {
            DatabaseType::SQLite => {
                let path = self.connection_string
                    .strip_prefix("sqlite:")
                    .unwrap_or(&self.connection_string);
                
                if path != ":memory:" {
                    // Check if directory exists
                    if let Some(parent) = Path::new(path).parent() {
                        if !parent.exists() {
                            anyhow::bail!("Directory does not exist: {}", parent.display());
                        }
                    }
                    
                    // Try to open/create the database
                    let conn = rusqlite::Connection::open(path)
                        .context("Failed to open SQLite database")?;
                    
                    // Test with a simple query
                    conn.execute_batch("SELECT 1")
                        .context("Failed to execute test query")?;
                }
                Ok(())
            }
            DatabaseType::PostgreSQL => {
                // For PostgreSQL, we need tokio runtime
                // For now, just validate the URL format
                if !self.connection_string.contains("://") {
                    anyhow::bail!("Invalid PostgreSQL connection string");
                }
                Ok(())
            }
        }
    }
}