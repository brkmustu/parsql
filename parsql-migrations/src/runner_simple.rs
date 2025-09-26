//! Migration runner implementation using simplified traits.

use crate::{
    config::MigrationConfig,
    error::{MigrationError, Result},
    traits_simple::{Migration, MigrationConnection},
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
            "test" => {
                // For testing, just execute a dummy query
                return conn.execute("CREATE TABLE IF NOT EXISTS parsql_migrations (version INT)");
            }
            db => return Err(MigrationError::Custom(format!("Unsupported database type: {}", db))),
        };
        
        conn.execute(&sql)?;
        Ok(())
    }
    
    /// Get all applied migrations
    fn get_applied_migrations(&self, conn: &mut dyn MigrationConnection) -> Result<MigrationMap> {
        let records = conn.query_migrations(&self.config.table.table_name)?;
        
        let mut map = MigrationMap::new();
        for record in records {
            let mut details = MigrationDetails::new(record.version, record.name);
            details.state = MigrationState::Applied;
            details.applied_at = Some(record.applied_at);
            details.checksum = record.checksum;
            details.execution_time_ms = record.execution_time_ms;
            
            map.insert(record.version, details);
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
        // If no migrations have been applied yet, there can't be gaps
        if applied.is_empty() {
            return Ok(());
        }
        
        // Find the highest applied version less than the current version
        let max_applied = applied
            .keys()
            .filter(|&&v| v < version)
            .max()
            .copied()
            .unwrap_or(0);
        
        // If max_applied is 0, it means all applied migrations have versions >= current version
        // This is fine, no gap check needed
        if max_applied == 0 {
            return Ok(());
        }
        
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
            conn.begin_transaction()?;
            let migration_result = migration.up(conn);
            let record_result = if migration_result.is_ok() {
                self.record_migration(conn, migration, start.elapsed().as_millis() as i64)
            } else {
                Ok(())
            };
            
            if migration_result.is_ok() && record_result.is_ok() {
                conn.commit_transaction()?;
                Ok(())
            } else {
                let _ = conn.rollback_transaction();
                migration_result?;
                record_result
            }
        } else {
            // Run without transaction
            let migration_result = migration.up(conn);
            let record_result = if migration_result.is_ok() {
                self.record_migration(conn, migration, start.elapsed().as_millis() as i64)
            } else {
                Ok(())
            };
            
            if migration_result.is_ok() && record_result.is_ok() {
                Ok(())
            } else {
                // Return the first error
                migration_result.and(record_result)
            }
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
            conn.begin_transaction()?;
            let migration_result = migration.down(conn);
            let record_result = if migration_result.is_ok() {
                self.remove_migration_record(conn, version)
            } else {
                Ok(())
            };
            
            if migration_result.is_ok() && record_result.is_ok() {
                conn.commit_transaction()?;
                Ok(())
            } else {
                let _ = conn.rollback_transaction();
                migration_result?;
                record_result
            }
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