//! Migration file creation utilities

use std::fs;
use std::path::PathBuf;
use chrono::Local;
use anyhow::{Context, Result};

pub struct MigrationCreator {
    migrations_dir: PathBuf,
}

impl MigrationCreator {
    pub fn new(migrations_dir: PathBuf) -> Self {
        Self { migrations_dir }
    }
    
    pub fn create_migration(&self, name: &str, migration_type: &str) -> Result<MigrationFiles> {
        // Create migrations directory if it doesn't exist
        fs::create_dir_all(&self.migrations_dir)
            .context("Failed to create migrations directory")?;
        
        // Generate timestamp-based version (compatible with CLI format)
        let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
        let safe_name = sanitize_name(name);
        
        match migration_type {
            "sql" => self.create_sql_migration(&timestamp, &safe_name),
            "rust" => self.create_rust_migration(&timestamp, &safe_name),
            _ => anyhow::bail!("Unsupported migration type: {}", migration_type),
        }
    }
    
    fn create_sql_migration(&self, version: &str, name: &str) -> Result<MigrationFiles> {
        // Use standardized naming format: {timestamp}_{name} (matching CLI after fix)
        let base_name = format!("{}_{}", version, name);
        let up_file = self.migrations_dir.join(format!("{}.up.sql", base_name));
        let down_file = self.migrations_dir.join(format!("{}.down.sql", base_name));
        
        // Create up migration template
        let up_content = format!(
            r#"-- Migration: {}
-- Version: {}
-- Created: {}

-- Add your UP migration SQL here
-- Example:
-- CREATE TABLE users (
--     id SERIAL PRIMARY KEY,
--     email VARCHAR(255) NOT NULL UNIQUE,
--     created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
-- );
"#,
            name,
            version,
            Local::now().format("%Y-%m-%d %H:%M:%S")
        );
        
        // Create down migration template
        let down_content = format!(
            r#"-- Migration: {} (rollback)
-- Version: {}
-- Created: {}

-- Add your DOWN migration SQL here
-- Example:
-- DROP TABLE IF EXISTS users;
"#,
            name,
            version,
            Local::now().format("%Y-%m-%d %H:%M:%S")
        );
        
        fs::write(&up_file, up_content)
            .context("Failed to create up migration file")?;
        fs::write(&down_file, down_content)
            .context("Failed to create down migration file")?;
        
        Ok(MigrationFiles {
            version: version.to_string(),
            name: name.to_string(),
            up_file: up_file.to_string_lossy().to_string(),
            down_file: Some(down_file.to_string_lossy().to_string()),
            migration_type: "sql".to_string(),
        })
    }
    
    fn create_rust_migration(&self, version: &str, name: &str) -> Result<MigrationFiles> {
        let file_name = format!("{}_{}.rs", version, name);
        let file_path = self.migrations_dir.join(&file_name);
        
        let content = format!(
            r#"//! Migration: {}
//! Version: {}
//! Created: {}

use parsql_migrations::{{Migration, MigrationConnection, MigrationError}};

pub struct Migration{};

impl Migration for Migration{} {{
    fn version(&self) -> i64 {{
        {}
    }}
    
    fn name(&self) -> &str {{
        "{}"
    }}
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {{
        // Add your UP migration logic here
        conn.execute(
            "CREATE TABLE users (
                id SERIAL PRIMARY KEY,
                email VARCHAR(255) NOT NULL UNIQUE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )"
        )?;
        
        Ok(())
    }}
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {{
        // Add your DOWN migration logic here
        conn.execute("DROP TABLE IF EXISTS users")?;
        Ok(())
    }}
}}
"#,
            name,
            version,
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            version,
            version,
            version,
            name
        );
        
        fs::write(&file_path, content)
            .context("Failed to create Rust migration file")?;
        
        // Update migrations/mod.rs if it exists
        self.update_mod_file(&file_name)?;
        
        Ok(MigrationFiles {
            version: version.to_string(),
            name: name.to_string(),
            up_file: file_path.to_string_lossy().to_string(),
            down_file: None,
            migration_type: "rust".to_string(),
        })
    }
    
    fn update_mod_file(&self, migration_file: &str) -> Result<()> {
        let mod_path = self.migrations_dir.join("mod.rs");
        let module_name = migration_file.trim_end_matches(".rs");
        
        if mod_path.exists() {
            let mut content = fs::read_to_string(&mod_path)?;
            content.push_str(&format!("\npub mod {};", module_name));
            fs::write(&mod_path, content)?;
        } else {
            let content = format!("//! Migration modules\n\npub mod {};", module_name);
            fs::write(&mod_path, content)?;
        }
        
        Ok(())
    }
}

pub struct MigrationFiles {
    pub version: String,
    pub name: String,
    pub up_file: String,
    pub down_file: Option<String>,
    pub migration_type: String,
}

fn sanitize_name(name: &str) -> String {
    // Use same sanitization as CLI for consistency
    name.replace(' ', "_").to_lowercase()
}