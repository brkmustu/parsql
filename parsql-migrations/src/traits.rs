//! Core traits for the migration system.

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

/// Database-agnostic connection trait for migrations
pub trait MigrationConnection: Send {
    /// Execute a SQL statement
    fn execute(&mut self, sql: &str) -> Result<()>;
    
    /// Execute a SQL query and return the number of affected rows
    fn execute_with_result(&mut self, sql: &str) -> Result<u64> {
        // Default implementation just executes and returns 0
        self.execute(sql)?;
        Ok(0)
    }
    
    /// Execute a query and return a single value
    fn query_one<T>(&mut self, sql: &str) -> Result<T>
    where
        T: FromSql;
    
    /// Execute a query and return multiple rows
    fn query<T>(&mut self, sql: &str) -> Result<Vec<T>>
    where
        T: FromSql;
    
    /// Begin a transaction
    fn transaction<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&mut dyn MigrationConnection) -> Result<R>;
        
    /// Get the database type (postgres, sqlite, etc.)
    fn database_type(&self) -> &str;
}

/// Trait for types that can be constructed from SQL query results
pub trait FromSql: Sized {
    /// Create an instance from a SQL row
    fn from_sql_row(row: &dyn SqlRow) -> Result<Self>;
}

/// Abstract representation of a database row
pub trait SqlRow {
    /// Get a value by column index
    fn get<T>(&self, idx: usize) -> Result<T>
    where
        T: FromSqlValue;
        
    /// Get a value by column name
    fn get_by_name<T>(&self, name: &str) -> Result<T>
    where
        T: FromSqlValue;
}

/// Trait for types that can be extracted from SQL values
pub trait FromSqlValue: Sized {
    /// Extract value from a SQL value
    fn from_sql_value(value: &dyn std::any::Any) -> Result<Self>;
}

// Async traits for async database support
#[cfg(any(feature = "tokio-postgres", feature = "deadpool-postgres"))]
use async_trait::async_trait;

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
    
    /// Execute a query and return a single value
    async fn query_one<T>(&mut self, sql: &str) -> Result<T>
    where
        T: FromSql + Send;
    
    /// Execute a query and return multiple rows
    async fn query<T>(&mut self, sql: &str) -> Result<Vec<T>>
    where
        T: FromSql + Send;
    
    /// Begin a transaction
    async fn transaction<F, R>(&mut self, f: F) -> Result<R>
    where
        F: for<'a> FnOnce(&'a mut dyn AsyncMigrationConnection) -> 
            std::pin::Pin<Box<dyn std::future::Future<Output = Result<R>> + Send + 'a>> + Send,
        R: Send;
        
    /// Get the database type
    fn database_type(&self) -> &str;
}

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

// Basic implementations for common types
impl FromSqlValue for i64 {
    fn from_sql_value(value: &dyn std::any::Any) -> Result<Self> {
        value.downcast_ref::<i64>()
            .copied()
            .ok_or_else(|| crate::MigrationError::Custom("Failed to convert to i64".into()))
    }
}

impl FromSqlValue for String {
    fn from_sql_value(value: &dyn std::any::Any) -> Result<Self> {
        value.downcast_ref::<String>()
            .cloned()
            .ok_or_else(|| crate::MigrationError::Custom("Failed to convert to String".into()))
    }
}

impl FromSqlValue for bool {
    fn from_sql_value(value: &dyn std::any::Any) -> Result<Self> {
        value.downcast_ref::<bool>()
            .copied()
            .ok_or_else(|| crate::MigrationError::Custom("Failed to convert to bool".into()))
    }
}

impl FromSqlValue for Option<String> {
    fn from_sql_value(value: &dyn std::any::Any) -> Result<Self> {
        if let Some(s) = value.downcast_ref::<String>() {
            Ok(Some(s.clone()))
        } else if value.downcast_ref::<()>().is_some() {
            Ok(None)
        } else {
            Err(crate::MigrationError::Custom("Failed to convert to Option<String>".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestMigration;
    
    impl Migration for TestMigration {
        fn version(&self) -> i64 { 1 }
        fn name(&self) -> &str { "test_migration" }
        
        fn up(&self, _conn: &mut dyn MigrationConnection) -> Result<()> {
            Ok(())
        }
        
        fn down(&self, _conn: &mut dyn MigrationConnection) -> Result<()> {
            Ok(())
        }
    }
    
    #[test]
    fn test_migration_checksum() {
        let migration = TestMigration;
        let checksum = migration.checksum();
        assert!(!checksum.is_empty());
        
        // Checksum should be consistent
        assert_eq!(checksum, migration.checksum());
    }
}