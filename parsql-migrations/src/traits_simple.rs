//! Simplified migration traits that avoid dyn compatibility issues.

use crate::error::Result;
use sha2::{Sha256, Digest};

/// Core trait for defining database migrations
pub trait Migration: Send + Sync {
    /// Get the unique version number for this migration
    fn version(&self) -> i64;
    
    /// Get the human-readable name for this migration
    fn name(&self) -> &str;
    
    /// Execute the migration (apply changes)
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<()>;
    
    /// Reverse the migration (rollback changes)
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<()>;
    
    /// Get the checksum of this migration for verification
    fn checksum(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.version().to_string());
        hasher.update(self.name());
        format!("{:x}", hasher.finalize())
    }
}

/// Database connection trait that is dyn compatible
pub trait MigrationConnection: Send {
    /// Execute a SQL statement
    fn execute(&mut self, sql: &str) -> Result<()>;
    
    /// Execute a SQL query and return the number of affected rows
    fn execute_with_result(&mut self, sql: &str) -> Result<u64> {
        // Default implementation just executes and returns 0
        self.execute(sql)?;
        Ok(0)
    }
    
    /// Get the database type (postgres, sqlite, etc.)
    fn database_type(&self) -> &str;
    
    /// Query for migration records
    fn query_migrations(&mut self, table_name: &str) -> Result<Vec<MigrationRecord>>;
    
    /// Begin a transaction - simple implementation
    fn begin_transaction(&mut self) -> Result<()> {
        self.execute("BEGIN")
    }
    
    /// Commit a transaction
    fn commit_transaction(&mut self) -> Result<()> {
        self.execute("COMMIT")
    }
    
    /// Rollback a transaction
    fn rollback_transaction(&mut self) -> Result<()> {
        self.execute("ROLLBACK")
    }
}

/// Record of an applied migration
#[derive(Debug, Clone)]
pub struct MigrationRecord {
    /// Version number of the migration
    pub version: i64,
    /// Name of the migration
    pub name: String,
    /// When the migration was applied
    pub applied_at: chrono::DateTime<chrono::Utc>,
    /// Checksum of the migration for verification
    pub checksum: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: Option<i64>,
}