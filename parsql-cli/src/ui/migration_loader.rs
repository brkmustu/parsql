//! Migration loading and execution utilities

use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use parsql_migrations::config::MigrationConfig;

pub struct MigrationLoader {
    migrations_dir: PathBuf,
    config: MigrationConfig,
}

impl MigrationLoader {
    pub fn new(migrations_dir: PathBuf, config: MigrationConfig) -> Self {
        Self {
            migrations_dir,
            config,
        }
    }
    
    /// Load all SQL migration files from the migrations directory
    pub fn load_sql_migrations(&self) -> Result<Vec<SqlMigration>> {
        let mut migrations = Vec::new();
        
        if !self.migrations_dir.exists() {
            return Ok(migrations);
        }
        
        // Read all files in migrations directory
        let entries = fs::read_dir(&self.migrations_dir)
            .context("Failed to read migrations directory")?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            // Only process .up.sql files
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                if filename.ends_with(".up.sql") {
                    // Extract version and name from filename
                    // Format: YYYYMMDDHHMMSS_name.up.sql
                    let base_name = filename.trim_end_matches(".up.sql");
                    
                    if let Some(underscore_pos) = base_name.find('_') {
                        let version_str = &base_name[..underscore_pos];
                        let name = &base_name[underscore_pos + 1..];
                        
                        if let Ok(version) = version_str.parse::<i64>() {
                            // Read up and down files
                            let up_content = fs::read_to_string(&path)
                                .context(format!("Failed to read {}", path.display()))?;
                            
                            let down_path = path.with_file_name(format!("{}.down.sql", base_name));
                            let down_content = if down_path.exists() {
                                Some(fs::read_to_string(&down_path)
                                    .context(format!("Failed to read {}", down_path.display()))?)
                            } else {
                                None
                            };
                            
                            migrations.push(SqlMigration {
                                version,
                                name: name.to_string(),
                                up_sql: up_content,
                                down_sql: down_content,
                                file_path: path.clone(),
                            });
                        }
                    }
                }
            }
        }
        
        // Sort by version
        migrations.sort_by_key(|m| m.version);
        
        Ok(migrations)
    }
    
    /// Get the status of all migrations (blocking version for TUI)
    pub fn get_migration_status_blocking(&self, db_url: &str) -> Result<Vec<MigrationStatus>> {
        let migrations = self.load_sql_migrations()?;
        let mut statuses = Vec::new();
        
        // Parse database URL to determine type
        if db_url.starts_with("sqlite:") {
            let path = db_url.strip_prefix("sqlite:").unwrap_or(db_url);
            if path != ":memory:" && std::path::Path::new(path).exists() {
                // Get applied migrations from database
                let applied = self.get_applied_migrations_sqlite(path)?;
                
                for migration in migrations {
                    let is_applied = applied.contains(&migration.version);
                    statuses.push(MigrationStatus {
                        version: migration.version,
                        name: migration.name,
                        applied: is_applied,
                        applied_at: if is_applied {
                            applied.iter()
                                .find(|&&v| v == migration.version)
                                .map(|_| chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
                        } else {
                            None
                        },
                    });
                }
            } else {
                // Database doesn't exist yet, all migrations are pending
                for migration in migrations {
                    statuses.push(MigrationStatus {
                        version: migration.version,
                        name: migration.name,
                        applied: false,
                        applied_at: None,
                    });
                }
            }
        } else if db_url.starts_with("postgresql://") || db_url.starts_with("postgres://") {
            // Get applied migrations from PostgreSQL database
            let applied = self.get_applied_migrations_postgres(db_url)?;
            
            for migration in migrations {
                let applied_info = applied.iter().find(|(v, _)| *v == migration.version);
                let is_applied = applied_info.is_some();
                
                statuses.push(MigrationStatus {
                    version: migration.version,
                    name: migration.name,
                    applied: is_applied,
                    applied_at: applied_info.map(|(_, timestamp)| timestamp.clone()),
                });
            }
        }
        
        Ok(statuses)
    }
    
    /// Get the status of all migrations (async version for CLI)
    pub async fn get_migration_status(&self, db_url: &str) -> Result<Vec<MigrationStatus>> {
        // For now, just call the blocking version
        // In the future, this could use proper async PostgreSQL
        self.get_migration_status_blocking(db_url)
    }
    
    /// Get applied migrations from SQLite database
    fn get_applied_migrations_sqlite(&self, db_path: &str) -> Result<Vec<i64>> {
        let conn = rusqlite::Connection::open(db_path)?;
        let mut applied = Vec::new();
        
        // Check if migrations table exists
        let table_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?)",
            [&self.config.table.table_name],
            |row| row.get(0),
        ).unwrap_or(false);
        
        if table_exists {
            let mut stmt = conn.prepare(&format!(
                "SELECT version FROM {} ORDER BY version",
                self.config.table.table_name
            ))?;
            
            let version_iter = stmt.query_map([], |row| {
                row.get::<_, i64>(0)
            })?;
            
            for version in version_iter {
                applied.push(version?);
            }
        }
        
        Ok(applied)
    }
    
    /// Get applied migrations from PostgreSQL database
    #[cfg(feature = "postgres")]
    fn get_applied_migrations_postgres(&self, db_url: &str) -> Result<Vec<(i64, String)>> {
        use postgres::{Client, NoTls};
        
        let mut client = Client::connect(db_url, NoTls)
            .context("Failed to connect to PostgreSQL database")?;
        
        let mut applied = Vec::new();
        
        // Check if migrations table exists
        let table_exists: bool = client.query_one(
            "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = $1)",
            &[&self.config.table.table_name],
        )?.get(0);
        
        if table_exists {
            let rows = client.query(
                &format!("SELECT version, applied_at FROM {} ORDER BY version", self.config.table.table_name),
                &[],
            )?;
            
            for row in rows {
                let version: i64 = row.get(0);
                let applied_at: std::time::SystemTime = row.get(1);
                let datetime: chrono::DateTime<chrono::Utc> = applied_at.into();
                let timestamp = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
                applied.push((version, timestamp));
            }
        }
        
        Ok(applied)
    }
    
    /// Get applied migrations from PostgreSQL database (fallback when postgres feature is disabled)
    #[cfg(not(feature = "postgres"))]
    fn get_applied_migrations_postgres(&self, _db_url: &str) -> Result<Vec<(i64, String)>> {
        Err(anyhow::anyhow!("PostgreSQL support not compiled in. Enable 'postgres' feature"))
    }
}

pub struct SqlMigration {
    pub version: i64,
    pub name: String,
    pub up_sql: String,
    pub down_sql: Option<String>,
    pub file_path: PathBuf,
}

pub struct MigrationStatus {
    pub version: i64,
    pub name: String,
    pub applied: bool,
    pub applied_at: Option<String>,
}