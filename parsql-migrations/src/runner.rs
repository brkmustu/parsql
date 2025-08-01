//! Migration runner implementation.

use crate::{
    config::MigrationConfig,
    error::{MigrationError, Result},
    traits::{FromSql, Migration, MigrationConnection, SqlRow},
    types::{MigrationDetails, MigrationMap, MigrationReport, MigrationResult, MigrationState, MigrationStatus},
};
use std::time::Instant;

/// The main migration runner
pub struct MigrationRunner {
    migrations: Vec<Box<dyn Migration>>,
    config: MigrationConfig,
}

impl MigrationRunner {
    /// Create a new migration runner with default configuration
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
            config: MigrationConfig::default(),
        }
    }
    
    /// Create a new migration runner with custom configuration
    pub fn with_config(config: MigrationConfig) -> Self {
        Self {
            migrations: Vec::new(),
            config,
        }
    }
    
    /// Add a migration to the runner
    pub fn add_migration(&mut self, migration: Box<dyn Migration>) -> &mut Self {
        self.migrations.push(migration);
        self
    }
    
    /// Add multiple migrations
    pub fn add_migrations(&mut self, migrations: Vec<Box<dyn Migration>>) -> &mut Self {
        self.migrations.extend(migrations);
        self
    }
    
    /// Get the configuration
    pub fn config(&self) -> &MigrationConfig {
        &self.config
    }
    
    /// Get mutable access to the configuration
    pub fn config_mut(&mut self) -> &mut MigrationConfig {
        &mut self.config
    }
    
    /// Run all pending migrations
    pub fn run(&mut self, conn: &mut dyn MigrationConnection) -> Result<MigrationReport> {
        let mut report = MigrationReport::new();
        
        // Ensure migrations table exists
        if self.config.auto_create_table {
            self.ensure_migration_table(conn)?;
        }
        
        // Sort migrations by version
        self.migrations.sort_by_key(|m| m.version());
        
        // Validate migrations
        self.validate_migrations()?;
        
        // Get applied migrations
        let applied = self.get_applied_migrations(conn)?;
        
        // Execute migrations
        for migration in &self.migrations {
            let version = migration.version();
            
            if applied.contains_key(&version) {
                // Skip already applied migrations
                report.add_skipped(version);
                continue;
            }
            
            // Check for gaps if not allowing out-of-order
            if !self.config.allow_out_of_order {
                self.check_migration_gap(version, &applied)?;
            }
            
            // Execute the migration
            let result = self.execute_migration(conn, migration.as_ref(), &mut report);
            
            if result.is_err() && self.config.stop_on_error {
                report.complete();
                return Ok(report);
            }
        }
        
        report.complete();
        Ok(report)
    }
    
    /// Rollback migrations to a specific version
    pub fn rollback(&mut self, conn: &mut dyn MigrationConnection, target_version: i64) -> Result<MigrationReport> {
        let mut report = MigrationReport::new();
        
        // Get applied migrations
        let applied = self.get_applied_migrations(conn)?;
        
        // Sort migrations by version (descending for rollback)
        self.migrations.sort_by_key(|m| std::cmp::Reverse(m.version()));
        
        // Execute rollbacks
        for migration in &self.migrations {
            let version = migration.version();
            
            if version <= target_version {
                // Stop when we reach the target version
                break;
            }
            
            if !applied.contains_key(&version) {
                // Skip migrations that haven't been applied
                continue;
            }
            
            // Execute the rollback
            let result = self.execute_rollback(conn, migration.as_ref(), &mut report);
            
            if result.is_err() && self.config.stop_on_error {
                report.complete();
                return Ok(report);
            }
        }
        
        report.complete();
        Ok(report)
    }
    
    /// Get the status of all migrations
    pub fn status(&self, conn: &mut dyn MigrationConnection) -> Result<Vec<MigrationStatus>> {
        let applied = self.get_applied_migrations(conn)?;
        
        let mut statuses = Vec::new();
        for migration in &self.migrations {
            let version = migration.version();
            let status = if let Some(details) = applied.get(&version) {
                MigrationStatus {
                    version,
                    name: migration.name().to_string(),
                    applied: true,
                    applied_at: details.applied_at,
                    execution_time_ms: details.execution_time_ms,
                }
            } else {
                MigrationStatus {
                    version,
                    name: migration.name().to_string(),
                    applied: false,
                    applied_at: None,
                    execution_time_ms: None,
                }
            };
            statuses.push(status);
        }
        
        // Sort by version
        statuses.sort_by_key(|s| s.version);
        
        Ok(statuses)
    }
    
    /// Ensure the migrations table exists
    fn ensure_migration_table(&self, conn: &mut dyn MigrationConnection) -> Result<()> {
        let sql = match conn.database_type() {
            "postgresql" | "postgres" => self.config.postgres_create_table_sql(),
            "sqlite" => self.config.sqlite_create_table_sql(),
            db => return Err(MigrationError::Custom(format!("Unsupported database type: {}", db))),
        };
        
        conn.execute(&sql)?;
        Ok(())
    }
    
    /// Get all applied migrations
    fn get_applied_migrations(&self, conn: &mut dyn MigrationConnection) -> Result<MigrationMap> {
        let sql = format!(
            "SELECT {}, {}, {}, {}, {} FROM {} ORDER BY {}",
            self.config.table.version_column,
            self.config.table.name_column,
            self.config.table.applied_at_column,
            self.config.table.checksum_column,
            self.config.table.execution_time_column,
            self.config.table.table_name,
            self.config.table.version_column
        );
        
        let rows: Vec<MigrationRow> = conn.query(&sql)?;
        
        let mut map = MigrationMap::new();
        for row in rows {
            let mut details = MigrationDetails::new(row.version, row.name);
            details.state = MigrationState::Applied;
            details.applied_at = Some(row.applied_at);
            details.checksum = row.checksum;
            details.execution_time_ms = row.execution_time_ms;
            
            map.insert(row.version, details);
        }
        
        Ok(map)
    }
    
    /// Validate migrations
    fn validate_migrations(&self) -> Result<()> {
        // Check for duplicate versions
        let mut versions = std::collections::HashSet::new();
        for migration in &self.migrations {
            let version = migration.version();
            if !versions.insert(version) {
                return Err(MigrationError::Custom(format!("Duplicate migration version: {}", version)));
            }
            
            // Validate version is positive
            if version <= 0 {
                return Err(MigrationError::InvalidVersion(version));
            }
        }
        
        Ok(())
    }
    
    /// Check for migration gaps
    fn check_migration_gap(&self, version: i64, applied: &MigrationMap) -> Result<()> {
        // Find the highest applied version less than the current version
        let max_applied = applied
            .keys()
            .filter(|&&v| v < version)
            .max()
            .copied()
            .unwrap_or(0);
        
        // Check if there are any unapplied migrations between max_applied and version
        for migration in &self.migrations {
            let v = migration.version();
            if v > max_applied && v < version && !applied.contains_key(&v) {
                return Err(MigrationError::MigrationGap(v));
            }
        }
        
        Ok(())
    }
    
    /// Execute a single migration
    fn execute_migration(
        &self,
        conn: &mut dyn MigrationConnection,
        migration: &dyn Migration,
        report: &mut MigrationReport,
    ) -> Result<()> {
        let version = migration.version();
        let name = migration.name();
        let start = Instant::now();
        
        println!("Executing migration {}: {}", version, name);
        
        let result = if self.config.transaction_per_migration {
            // Run in transaction
            conn.transaction(|tx| {
                migration.up(tx)?;
                self.record_migration(tx, migration, start.elapsed().as_millis() as i64)?;
                Ok(())
            })
        } else {
            // Run without transaction
            migration.up(conn)?;
            self.record_migration(conn, migration, start.elapsed().as_millis() as i64)?;
            Ok(())
        };
        
        let execution_time = start.elapsed().as_millis() as i64;
        
        match result {
            Ok(()) => {
                report.add_success(MigrationResult::success(version, name.to_string(), execution_time));
                println!("  ✓ Migration {} completed in {}ms", version, execution_time);
                Ok(())
            }
            Err(e) => {
                report.add_failure(MigrationResult::failure(version, name.to_string(), e.to_string(), execution_time));
                println!("  ✗ Migration {} failed: {}", version, e);
                Err(e)
            }
        }
    }
    
    /// Execute a single rollback
    fn execute_rollback(
        &self,
        conn: &mut dyn MigrationConnection,
        migration: &dyn Migration,
        report: &mut MigrationReport,
    ) -> Result<()> {
        let version = migration.version();
        let name = migration.name();
        let start = Instant::now();
        
        println!("Rolling back migration {}: {}", version, name);
        
        let result = if self.config.transaction_per_migration {
            // Run in transaction
            conn.transaction(|tx| {
                migration.down(tx)?;
                self.remove_migration_record(tx, version)?;
                Ok(())
            })
        } else {
            // Run without transaction
            migration.down(conn)?;
            self.remove_migration_record(conn, version)?;
            Ok(())
        };
        
        let execution_time = start.elapsed().as_millis() as i64;
        
        match result {
            Ok(()) => {
                report.add_success(MigrationResult::success(version, name.to_string(), execution_time));
                println!("  ✓ Rollback {} completed in {}ms", version, execution_time);
                Ok(())
            }
            Err(e) => {
                report.add_failure(MigrationResult::failure(version, name.to_string(), e.to_string(), execution_time));
                println!("  ✗ Rollback {} failed: {}", version, e);
                Err(e)
            }
        }
    }
    
    /// Record a successful migration
    fn record_migration(
        &self,
        conn: &mut dyn MigrationConnection,
        migration: &dyn Migration,
        execution_time_ms: i64,
    ) -> Result<()> {
        let sql = format!(
            "INSERT INTO {} ({}, {}, {}, {}) VALUES ({}, '{}', '{}', {})",
            self.config.table.table_name,
            self.config.table.version_column,
            self.config.table.name_column,
            self.config.table.checksum_column,
            self.config.table.execution_time_column,
            migration.version(),
            migration.name().replace('\'', "''"),
            migration.checksum(),
            execution_time_ms
        );
        
        conn.execute(&sql)?;
        Ok(())
    }
    
    /// Remove a migration record (for rollback)
    fn remove_migration_record(&self, conn: &mut dyn MigrationConnection, version: i64) -> Result<()> {
        let sql = format!(
            "DELETE FROM {} WHERE {} = {}",
            self.config.table.table_name,
            self.config.table.version_column,
            version
        );
        
        conn.execute(&sql)?;
        Ok(())
    }
}

impl Default for MigrationRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal struct for migration row results
struct MigrationRow {
    version: i64,
    name: String,
    applied_at: chrono::DateTime<chrono::Utc>,
    checksum: Option<String>,
    execution_time_ms: Option<i64>,
}

impl FromSql for MigrationRow {
    fn from_sql_row(row: &dyn SqlRow) -> Result<Self> {
        Ok(Self {
            version: row.get(0)?,
            name: row.get(1)?,
            applied_at: row.get(2)?,
            checksum: row.get(3)?,
            execution_time_ms: row.get(4)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_runner_creation() {
        let runner = MigrationRunner::new();
        assert_eq!(runner.migrations.len(), 0);
        assert!(runner.config.transaction_per_migration);
        
        let config = MigrationConfig::new().with_transactions(false);
        let runner = MigrationRunner::with_config(config);
        assert!(!runner.config.transaction_per_migration);
    }
}