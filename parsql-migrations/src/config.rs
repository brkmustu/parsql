//! Configuration options for the migration system.

use crate::types::TableConfig;
use std::time::Duration;

/// Configuration for the migration runner
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Table configuration
    pub table: TableConfig,
    
    /// Whether to run each migration in a transaction
    pub transaction_per_migration: bool,
    
    /// Lock timeout for acquiring exclusive migration lock
    pub lock_timeout: Option<Duration>,
    
    /// Whether to verify checksums of applied migrations
    pub verify_checksums: bool,
    
    /// Whether to allow out-of-order migrations
    pub allow_out_of_order: bool,
    
    /// Whether to create the migrations table if it doesn't exist
    pub auto_create_table: bool,
    
    /// Maximum number of retries for transient errors
    pub max_retries: u32,
    
    /// Delay between retries
    pub retry_delay: Duration,
    
    /// Whether to continue on error or stop
    pub stop_on_error: bool,
    
    /// Custom SQL for creating the migrations table (database-specific)
    pub create_table_sql: Option<String>,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            table: TableConfig::default(),
            transaction_per_migration: true,
            lock_timeout: Some(Duration::from_secs(10)),
            verify_checksums: true,
            allow_out_of_order: false,
            auto_create_table: true,
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            stop_on_error: true,
            create_table_sql: None,
        }
    }
}

impl MigrationConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the table name for migrations
    pub fn with_table_name(mut self, name: impl Into<String>) -> Self {
        self.table.table_name = name.into();
        self
    }
    
    /// Enable or disable transactions per migration
    pub fn with_transactions(mut self, enabled: bool) -> Self {
        self.transaction_per_migration = enabled;
        self
    }
    
    /// Set the lock timeout
    pub fn with_lock_timeout(mut self, timeout: Duration) -> Self {
        self.lock_timeout = Some(timeout);
        self
    }
    
    /// Disable lock timeout
    pub fn without_lock_timeout(mut self) -> Self {
        self.lock_timeout = None;
        self
    }
    
    /// Enable or disable checksum verification
    pub fn with_checksum_verification(mut self, enabled: bool) -> Self {
        self.verify_checksums = enabled;
        self
    }
    
    /// Allow out-of-order migrations
    pub fn allow_out_of_order(mut self, enabled: bool) -> Self {
        self.allow_out_of_order = enabled;
        self
    }
    
    /// Set whether to auto-create the migrations table
    pub fn with_auto_create_table(mut self, enabled: bool) -> Self {
        self.auto_create_table = enabled;
        self
    }
    
    /// Set maximum retries for transient errors
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }
    
    /// Set delay between retries
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }
    
    /// Set whether to stop on error
    pub fn stop_on_error(mut self, enabled: bool) -> Self {
        self.stop_on_error = enabled;
        self
    }
    
    /// Set custom create table SQL
    pub fn with_create_table_sql(mut self, sql: impl Into<String>) -> Self {
        self.create_table_sql = Some(sql.into());
        self
    }
    
    /// Get the SQL for creating the migrations table for PostgreSQL
    pub fn postgres_create_table_sql(&self) -> String {
        if let Some(ref sql) = self.create_table_sql {
            return sql.clone();
        }
        
        format!(
            r#"CREATE TABLE IF NOT EXISTS {} (
                {} BIGINT PRIMARY KEY,
                {} VARCHAR(255) NOT NULL,
                {} TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                {} VARCHAR(64),
                {} BIGINT,
                {} BOOLEAN NOT NULL DEFAULT TRUE,
                {} TIMESTAMP
            )"#,
            self.table.table_name,
            self.table.version_column,
            self.table.name_column,
            self.table.applied_at_column,
            self.table.checksum_column,
            self.table.execution_time_column,
            "success",
            self.table.rolled_back_at_column
        )
    }
    
    /// Get the SQL for creating the migrations table for SQLite
    pub fn sqlite_create_table_sql(&self) -> String {
        if let Some(ref sql) = self.create_table_sql {
            return sql.clone();
        }
        
        format!(
            r#"CREATE TABLE IF NOT EXISTS {} (
                {} INTEGER PRIMARY KEY,
                {} TEXT NOT NULL,
                {} TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                {} TEXT,
                {} INTEGER,
                {} INTEGER NOT NULL DEFAULT 1,
                {} TEXT
            )"#,
            self.table.table_name,
            self.table.version_column,
            self.table.name_column,
            self.table.applied_at_column,
            self.table.checksum_column,
            self.table.execution_time_column,
            "success",
            self.table.rolled_back_at_column
        )
    }
}

/// Builder for migration configuration
pub struct MigrationConfigBuilder {
    config: MigrationConfig,
}

impl MigrationConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: MigrationConfig::default(),
        }
    }
    
    /// Set the table name
    pub fn table_name(mut self, name: impl Into<String>) -> Self {
        self.config.table.table_name = name.into();
        self
    }
    
    /// Enable transactions
    pub fn with_transactions(mut self) -> Self {
        self.config.transaction_per_migration = true;
        self
    }
    
    /// Disable transactions
    pub fn without_transactions(mut self) -> Self {
        self.config.transaction_per_migration = false;
        self
    }
    
    /// Set lock timeout
    pub fn lock_timeout(mut self, timeout: Duration) -> Self {
        self.config.lock_timeout = Some(timeout);
        self
    }
    
    /// Enable checksum verification
    pub fn verify_checksums(mut self) -> Self {
        self.config.verify_checksums = true;
        self
    }
    
    /// Disable checksum verification
    pub fn skip_checksum_verification(mut self) -> Self {
        self.config.verify_checksums = false;
        self
    }
    
    /// Allow out-of-order migrations
    pub fn allow_out_of_order(mut self) -> Self {
        self.config.allow_out_of_order = true;
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> MigrationConfig {
        self.config
    }
}

impl Default for MigrationConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_builder() {
        let config = MigrationConfigBuilder::new()
            .table_name("custom_migrations")
            .without_transactions()
            .lock_timeout(Duration::from_secs(30))
            .skip_checksum_verification()
            .allow_out_of_order()
            .build();
        
        assert_eq!(config.table.table_name, "custom_migrations");
        assert!(!config.transaction_per_migration);
        assert_eq!(config.lock_timeout, Some(Duration::from_secs(30)));
        assert!(!config.verify_checksums);
        assert!(config.allow_out_of_order);
    }
    
    #[test]
    fn test_create_table_sql() {
        let config = MigrationConfig::new();
        
        let pg_sql = config.postgres_create_table_sql();
        assert!(pg_sql.contains("CREATE TABLE IF NOT EXISTS parsql_migrations"));
        assert!(pg_sql.contains("BIGINT PRIMARY KEY"));
        assert!(pg_sql.contains("TIMESTAMP"));
        
        let sqlite_sql = config.sqlite_create_table_sql();
        assert!(sqlite_sql.contains("CREATE TABLE IF NOT EXISTS parsql_migrations"));
        assert!(sqlite_sql.contains("INTEGER PRIMARY KEY"));
        assert!(sqlite_sql.contains("TEXT"));
    }
}