//! Simplified traits for the migration system to avoid dyn compatibility issues.

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

/// Simplified database connection trait that is dyn compatible
pub trait MigrationConnection: Send {
    /// Execute a SQL statement
    fn execute(&mut self, sql: &str) -> Result<()>;
    
    /// Execute a SQL query and return the number of affected rows
    fn execute_with_result(&mut self, sql: &str) -> Result<u64> {
        // Default implementation just executes and returns 0
        self.execute(sql)?;
        Ok(0)
    }
    
    /// Begin a transaction and execute a closure
    fn with_transaction<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut dyn MigrationConnection) -> Result<()>;
        
    /// Get the database type (postgres, sqlite, etc.)
    fn database_type(&self) -> &str;
    
    /// Query for migration records (database-specific implementation)
    fn query_migrations(&mut self, table_name: &str) -> Result<Vec<MigrationRecord>>;
}

/// Record of an applied migration
#[derive(Debug, Clone)]
pub struct MigrationRecord {
    pub version: i64,
    pub name: String,
    pub applied_at: chrono::DateTime<chrono::Utc>,
    pub checksum: Option<String>,
    pub execution_time_ms: Option<i64>,
}

// Async traits for async database support
#[cfg(any(feature = "tokio-postgres", feature = "deadpool-postgres"))]
use async_trait::async_trait;

/// Async version of Migration trait
#[cfg(any(feature = "tokio-postgres", feature = "deadpool-postgres"))]
#[async_trait]
pub trait AsyncMigration: Send + Sync {
    /// Get the unique version number
    fn version(&self) -> i64;
    
    /// Get the human-readable name
    fn name(&self) -> &str;
    
    /// Execute the migration asynchronously
    async fn up(&self, conn: &mut dyn AsyncMigrationConnection) -> Result<()>;
    
    /// Reverse the migration asynchronously
    async fn down(&self, conn: &mut dyn AsyncMigrationConnection) -> Result<()>;
    
    /// Get the checksum
    fn checksum(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.version().to_string());
        hasher.update(self.name());
        format!("{:x}", hasher.finalize())
    }
}

/// Async version of MigrationConnection
#[cfg(any(feature = "tokio-postgres", feature = "deadpool-postgres"))]
#[async_trait]
pub trait AsyncMigrationConnection: Send {
    /// Execute a SQL statement asynchronously
    async fn execute(&mut self, sql: &str) -> Result<()>;
    
    /// Execute a SQL query and return the number of affected rows
    async fn execute_with_result(&mut self, sql: &str) -> Result<u64> {
        self.execute(sql).await?;
        Ok(0)
    }
    
    /// Begin a transaction
    async fn with_transaction<'a>(&'a mut self, 
        f: Box<dyn FnOnce(&'a mut dyn AsyncMigrationConnection) -> 
            std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> + Send + 'a>
    ) -> Result<()>;
        
    /// Get the database type
    fn database_type(&self) -> &str;
    
    /// Query for migration records
    async fn query_migrations(&mut self, table_name: &str) -> Result<Vec<MigrationRecord>>;
}