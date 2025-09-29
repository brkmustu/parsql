//! Common test utilities for parsql-cli integration tests

use anyhow::Result;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Set up a temporary test environment with migrations directory
pub fn setup_test_env() -> Result<TempDir> {
    let temp_dir = tempfile::tempdir()?;
    
    // Create migrations directory
    let migrations_dir = temp_dir.path().join("migrations");
    fs::create_dir_all(&migrations_dir)?;
    
    Ok(temp_dir)
}

/// Create a test migration with realistic SQL content
pub fn create_test_migration(directory: &str, name: &str, migration_type: &str) -> Result<()> {
    // Use nanoseconds to avoid timestamp collisions in tests
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) / 1_000_000;
    let dir_path = Path::new(directory);
    fs::create_dir_all(dir_path)?;
    
    let safe_name = name.replace(' ', "_").to_lowercase();
    
    match migration_type {
        "sql" => {
            let up_file = dir_path.join(format!("{}_{}.up.sql", timestamp, safe_name));
            let down_file = dir_path.join(format!("{}_{}.down.sql", timestamp, safe_name));
            
            let table_name = safe_name.replace("create_", "").replace("add_", "");
            
            let up_content = format!(
                "-- Migration: {}\n-- Version: {}\n-- Created: {}\n\n{}\n",
                name,
                timestamp,
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                get_up_sql_for_table(&table_name)
            );
            
            let down_content = format!(
                "-- Migration: {} (rollback)\n-- Version: {}\n-- Created: {}\n\n{}\n",
                name,
                timestamp,
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                get_down_sql_for_table(&table_name)
            );
            
            fs::write(&up_file, up_content)?;
            fs::write(&down_file, down_content)?;
        }
        
        _ => anyhow::bail!("Unknown migration type: {}", migration_type),
    }
    
    Ok(())
}

/// Generate realistic UP SQL for a table based on its name
fn get_up_sql_for_table(table_name: &str) -> String {
    match table_name {
        "users" => {
            r#"CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);"#.to_string()
        }
        "posts" => {
            r#"CREATE TABLE posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    content TEXT,
    user_id INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);"#.to_string()
        }
        "comments" => {
            r#"CREATE TABLE comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    post_id INTEGER,
    user_id INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (post_id) REFERENCES posts(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);"#.to_string()
        }
        "categories" => {
            r#"CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);"#.to_string()
        }
        _ => {
            format!(
                r#"CREATE TABLE {} (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);"#,
                table_name
            )
        }
    }
}

/// Generate realistic DOWN SQL for a table
fn get_down_sql_for_table(table_name: &str) -> String {
    format!("DROP TABLE IF EXISTS {};", table_name)
}

/// Assert that a migration was applied by checking if table exists
pub fn assert_migration_applied(database_url: &str, table_name: &str) -> Result<()> {
    use rusqlite::Connection;
    
    let db_path = database_url.strip_prefix("sqlite:").unwrap_or(database_url);
    let conn = Connection::open(db_path)?;
    
    // Check if table exists
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?1")?;
    let exists = stmt.exists([table_name])?;
    
    if !exists {
        anyhow::bail!("Table '{}' does not exist, migration was not applied", table_name);
    }
    
    Ok(())
}

/// Assert that a migration was rolled back by checking if table doesn't exist
pub fn assert_migration_rolled_back(database_url: &str, table_name: &str) -> Result<()> {
    use rusqlite::Connection;
    
    let db_path = database_url.strip_prefix("sqlite:").unwrap_or(database_url);
    let conn = Connection::open(db_path)?;
    
    // Check if table exists
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?1")?;
    let exists = stmt.exists([table_name])?;
    
    if exists {
        anyhow::bail!("Table '{}' still exists, migration was not rolled back", table_name);
    }
    
    Ok(())
}

/// Create a test configuration with custom settings
pub fn create_test_config(migrations_dir: &str) -> parsql_cli::config::Config {
    parsql_cli::config::Config::default_with_directory(migrations_dir)
}

/// Count migration files in a directory
pub fn count_migration_files(directory: &str) -> Result<usize> {
    let migrations_dir = Path::new(directory);
    
    if !migrations_dir.exists() {
        return Ok(0);
    }
    
    let count = fs::read_dir(migrations_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let binding = entry.file_name();
            let file_name = binding.to_string_lossy();
            file_name.ends_with(".up.sql") || file_name.ends_with(".down.sql")
        })
        .count();
    
    Ok(count)
}

/// Get all migration versions from directory
pub fn get_migration_versions(directory: &str) -> Result<Vec<i64>> {
    let migrations_dir = Path::new(directory);
    let mut versions = Vec::new();
    
    if !migrations_dir.exists() {
        return Ok(versions);
    }
    
    for entry in fs::read_dir(migrations_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        
        if file_name.ends_with(".up.sql") {
            let parts: Vec<&str> = file_name.splitn(2, '_').collect();
            if let Ok(version) = parts[0].parse::<i64>() {
                versions.push(version);
            }
        }
    }
    
    versions.sort();
    versions.dedup();
    Ok(versions)
}