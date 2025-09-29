//! Migration execution utilities

use anyhow::{Context, Result};
use std::time::Instant;
use super::migration_loader::SqlMigration;
use super::output_stream::OutputStreamWidget;
use parsql_migrations::config::MigrationConfig;

pub struct MigrationExecutor {
    config: MigrationConfig,
}

impl MigrationExecutor {
    pub fn new(config: MigrationConfig) -> Self {
        Self { config }
    }
    
    /// Run pending migrations for SQLite
    pub fn run_sqlite_migrations(
        &self,
        db_path: &str,
        migrations: Vec<SqlMigration>,
        output: &mut OutputStreamWidget,
    ) -> Result<usize> {
        output.add_info(format!("Connecting to SQLite database: {}", db_path));
        
        let mut conn = rusqlite::Connection::open(db_path)
            .context("Failed to open SQLite database")?;
        
        // Create migrations table if it doesn't exist
        self.ensure_migrations_table_sqlite(&conn)?;
        
        // Get already applied migrations
        let applied = self.get_applied_versions_sqlite(&conn)?;
        
        let mut applied_count = 0;
        
        for migration in migrations {
            if applied.contains(&migration.version) {
                output.add_info(format!("Skipping already applied migration: {} - {}", 
                    migration.version, migration.name));
                continue;
            }
            
            output.add_progress(format!("Running migration: {} - {}", 
                migration.version, migration.name));
            
            let start = Instant::now();
            
            // Execute migration in transaction if configured
            if self.config.transaction_per_migration {
                let tx = conn.transaction()?;
                
                // Execute the migration SQL
                tx.execute_batch(&migration.up_sql)
                    .context(format!("Failed to execute migration {}", migration.version))?;
                
                let execution_time = start.elapsed();
                
                // Record the migration
                tx.execute(
                    &format!(
                        "INSERT INTO {} (version, name, checksum, applied_at, execution_time_ms) VALUES (?1, ?2, ?3, datetime('now'), ?4)",
                        self.config.table.table_name
                    ),
                    rusqlite::params![
                        migration.version,
                        migration.name,
                        calculate_checksum(&migration.up_sql),
                        execution_time.as_millis() as i64,
                    ],
                )?;
                
                tx.commit()?;
            } else {
                // Execute without transaction
                conn.execute_batch(&migration.up_sql)
                    .context(format!("Failed to execute migration {}", migration.version))?;
                
                let execution_time = start.elapsed();
                
                conn.execute(
                    &format!(
                        "INSERT INTO {} (version, name, checksum, applied_at, execution_time_ms) VALUES (?1, ?2, ?3, datetime('now'), ?4)",
                        self.config.table.table_name
                    ),
                    rusqlite::params![
                        migration.version,
                        migration.name,
                        calculate_checksum(&migration.up_sql),
                        execution_time.as_millis() as i64,
                    ],
                )?;
            }
            
            let elapsed = start.elapsed();
            output.add_success(format!(
                "Applied migration {} - {} ({:.2}ms)", 
                migration.version, 
                migration.name,
                elapsed.as_secs_f64() * 1000.0
            ));
            
            applied_count += 1;
        }
        
        Ok(applied_count)
    }
    
    /// Rollback to a specific version for SQLite
    pub fn rollback_sqlite(
        &self,
        db_path: &str,
        target_version: i64,
        migrations: Vec<SqlMigration>,
        output: &mut OutputStreamWidget,
    ) -> Result<usize> {
        output.add_info(format!("Connecting to SQLite database: {}", db_path));
        
        let mut conn = rusqlite::Connection::open(db_path)
            .context("Failed to open SQLite database")?;
        
        // Get applied migrations in reverse order
        let applied = self.get_applied_versions_sqlite(&conn)?;
        let mut to_rollback = Vec::new();
        
        for version in applied.iter().rev() {
            if *version > target_version {
                if let Some(migration) = migrations.iter().find(|m| m.version == *version) {
                    if migration.down_sql.is_some() {
                        to_rollback.push(migration);
                    } else {
                        output.add_warning(format!(
                            "Migration {} has no down script, skipping rollback", 
                            version
                        ));
                    }
                }
            }
        }
        
        let mut rolled_back = 0;
        
        for migration in to_rollback {
            output.add_progress(format!("Rolling back migration: {} - {}", 
                migration.version, migration.name));
            
            let start = Instant::now();
            
            if let Some(down_sql) = &migration.down_sql {
                if self.config.transaction_per_migration {
                    let tx = conn.transaction()?;
                    
                    tx.execute_batch(down_sql)
                        .context(format!("Failed to rollback migration {}", migration.version))?;
                    
                    tx.execute(
                        &format!("DELETE FROM {} WHERE version = ?1", self.config.table.table_name),
                        [migration.version],
                    )?;
                    
                    tx.commit()?;
                } else {
                    conn.execute_batch(down_sql)
                        .context(format!("Failed to rollback migration {}", migration.version))?;
                    
                    conn.execute(
                        &format!("DELETE FROM {} WHERE version = ?1", self.config.table.table_name),
                        [migration.version],
                    )?;
                }
                
                let elapsed = start.elapsed();
                output.add_success(format!(
                    "Rolled back migration {} - {} ({:.2}ms)", 
                    migration.version, 
                    migration.name,
                    elapsed.as_secs_f64() * 1000.0
                ));
                
                rolled_back += 1;
            }
        }
        
        Ok(rolled_back)
    }
    
    /// Ensure migrations table exists in SQLite
    fn ensure_migrations_table_sqlite(&self, conn: &rusqlite::Connection) -> Result<()> {
        let create_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                checksum TEXT,
                applied_at TEXT NOT NULL,
                execution_time_ms INTEGER
            )
            "#,
            self.config.table.table_name
        );
        
        conn.execute_batch(&create_sql)
            .context("Failed to create migrations table")?;
        
        Ok(())
    }
    
    /// Get applied migration versions from SQLite
    fn get_applied_versions_sqlite(&self, conn: &rusqlite::Connection) -> Result<Vec<i64>> {
        let mut applied = Vec::new();
        
        // Check if table exists first
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
    
    /// Run pending migrations for PostgreSQL
    #[cfg(feature = "postgres")]
    pub fn run_postgres_migrations(
        &self,
        db_url: &str,
        migrations: Vec<SqlMigration>,
        output: &mut OutputStreamWidget,
    ) -> Result<usize> {
        use postgres::{Client, NoTls};
        
        output.add_info(format!("Connecting to PostgreSQL database"));
        
        let mut client = Client::connect(db_url, NoTls)
            .context("Failed to connect to PostgreSQL database")?;
        
        // Create migrations table if it doesn't exist
        self.ensure_migrations_table_postgres(&mut client)?;
        
        // Get already applied migrations
        let applied = self.get_applied_versions_postgres(&mut client)?;
        
        let mut applied_count = 0;
        
        for migration in migrations {
            if applied.contains(&migration.version) {
                output.add_info(format!("Skipping already applied migration: {} - {}", 
                    migration.version, migration.name));
                continue;
            }
            
            output.add_progress(format!("Running migration: {} - {}", 
                migration.version, migration.name));
            
            let start = Instant::now();
            
            // Execute migration in transaction if configured
            if self.config.transaction_per_migration {
                let mut tx = client.transaction()?;
                
                // Execute the migration SQL
                tx.batch_execute(&migration.up_sql)
                    .context(format!("Failed to execute migration {}", migration.version))?;
                
                let execution_time = start.elapsed();
                
                // Record the migration
                tx.execute(
                    &format!(
                        "INSERT INTO {} (version, name, checksum, applied_at, execution_time_ms) VALUES ($1, $2, $3, NOW(), $4)",
                        self.config.table.table_name
                    ),
                    &[
                        &migration.version,
                        &migration.name,
                        &calculate_checksum(&migration.up_sql),
                        &(execution_time.as_millis() as i64),
                    ],
                )?;
                
                tx.commit()?;
            } else {
                // Execute without transaction
                client.batch_execute(&migration.up_sql)
                    .context(format!("Failed to execute migration {}", migration.version))?;
                
                let execution_time = start.elapsed();
                
                client.execute(
                    &format!(
                        "INSERT INTO {} (version, name, checksum, applied_at, execution_time_ms) VALUES ($1, $2, $3, NOW(), $4)",
                        self.config.table.table_name
                    ),
                    &[
                        &migration.version,
                        &migration.name,
                        &calculate_checksum(&migration.up_sql),
                        &(execution_time.as_millis() as i64),
                    ],
                )?;
            }
            
            let elapsed = start.elapsed();
            output.add_success(format!(
                "Applied migration {} - {} ({:.2}ms)", 
                migration.version, 
                migration.name,
                elapsed.as_secs_f64() * 1000.0
            ));
            
            applied_count += 1;
        }
        
        Ok(applied_count)
    }
    
    /// Rollback to a specific version for PostgreSQL
    #[cfg(feature = "postgres")]
    pub fn rollback_postgres(
        &self,
        db_url: &str,
        target_version: i64,
        migrations: Vec<SqlMigration>,
        output: &mut OutputStreamWidget,
    ) -> Result<usize> {
        use postgres::{Client, NoTls};
        
        output.add_info(format!("Connecting to PostgreSQL database"));
        
        let mut client = Client::connect(db_url, NoTls)
            .context("Failed to connect to PostgreSQL database")?;
        
        // Get applied migrations in reverse order
        let applied = self.get_applied_versions_postgres(&mut client)?;
        let mut to_rollback = Vec::new();
        
        for version in applied.iter().rev() {
            if *version > target_version {
                if let Some(migration) = migrations.iter().find(|m| m.version == *version) {
                    if migration.down_sql.is_some() {
                        to_rollback.push(migration);
                    } else {
                        output.add_warning(format!(
                            "Migration {} has no down script, skipping rollback", 
                            version
                        ));
                    }
                }
            }
        }
        
        let mut rolled_back = 0;
        
        for migration in to_rollback {
            output.add_progress(format!("Rolling back migration: {} - {}", 
                migration.version, migration.name));
            
            let start = Instant::now();
            
            if let Some(down_sql) = &migration.down_sql {
                if self.config.transaction_per_migration {
                    let mut tx = client.transaction()?;
                    
                    tx.batch_execute(down_sql)
                        .context(format!("Failed to rollback migration {}", migration.version))?;
                    
                    tx.execute(
                        &format!("DELETE FROM {} WHERE version = $1", self.config.table.table_name),
                        &[&migration.version],
                    )?;
                    
                    tx.commit()?;
                } else {
                    client.batch_execute(down_sql)
                        .context(format!("Failed to rollback migration {}", migration.version))?;
                    
                    client.execute(
                        &format!("DELETE FROM {} WHERE version = $1", self.config.table.table_name),
                        &[&migration.version],
                    )?;
                }
                
                let elapsed = start.elapsed();
                output.add_success(format!(
                    "Rolled back migration {} - {} ({:.2}ms)", 
                    migration.version, 
                    migration.name,
                    elapsed.as_secs_f64() * 1000.0
                ));
                
                rolled_back += 1;
            }
        }
        
        Ok(rolled_back)
    }
    
    /// Ensure migrations table exists in PostgreSQL
    #[cfg(feature = "postgres")]
    fn ensure_migrations_table_postgres(&self, client: &mut postgres::Client) -> Result<()> {
        let create_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                version BIGINT PRIMARY KEY,
                name TEXT NOT NULL,
                checksum TEXT,
                applied_at TIMESTAMP NOT NULL,
                execution_time_ms BIGINT
            )
            "#,
            self.config.table.table_name
        );
        
        client.batch_execute(&create_sql)
            .context("Failed to create migrations table")?;
        
        Ok(())
    }
    
    /// Get applied migration versions from PostgreSQL
    #[cfg(feature = "postgres")]
    fn get_applied_versions_postgres(&self, client: &mut postgres::Client) -> Result<Vec<i64>> {
        let mut applied = Vec::new();
        
        // Check if table exists first
        let table_exists: bool = client.query_one(
            "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = $1)",
            &[&self.config.table.table_name],
        )?.get(0);
        
        if table_exists {
            let rows = client.query(
                &format!("SELECT version FROM {} ORDER BY version", self.config.table.table_name),
                &[],
            )?;
            
            for row in rows {
                applied.push(row.get::<_, i64>(0));
            }
        }
        
        Ok(applied)
    }
    
    /// Fallback methods when postgres feature is disabled
    #[cfg(not(feature = "postgres"))]
    pub fn run_postgres_migrations(
        &self,
        _db_url: &str,
        _migrations: Vec<SqlMigration>,
        output: &mut OutputStreamWidget,
    ) -> Result<usize> {
        output.add_error("PostgreSQL support not compiled in. Enable 'postgres' feature".to_string());
        Err(anyhow::anyhow!("PostgreSQL support not compiled in. Enable 'postgres' feature"))
    }
    
    #[cfg(not(feature = "postgres"))]
    pub fn rollback_postgres(
        &self,
        _db_url: &str,
        _target_version: i64,
        _migrations: Vec<SqlMigration>,
        output: &mut OutputStreamWidget,
    ) -> Result<usize> {
        output.add_error("PostgreSQL support not compiled in. Enable 'postgres' feature".to_string());
        Err(anyhow::anyhow!("PostgreSQL support not compiled in. Enable 'postgres' feature"))
    }
}

fn calculate_checksum(content: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}