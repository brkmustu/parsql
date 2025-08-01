//! Error types for the migration system.

use thiserror::Error;

/// Main error type for migration operations
#[derive(Debug, Error)]
pub enum MigrationError {
    /// Database operation failed
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    /// Migration has already been applied
    #[error("Migration {0} has already been applied")]
    AlreadyApplied(i64),
    
    /// Migration was not found
    #[error("Migration {0} not found")]
    NotFound(i64),
    
    /// Checksum verification failed
    #[error("Checksum mismatch for migration {version}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        /// Migration version
        version: i64,
        /// Expected checksum
        expected: String,
        /// Actual checksum
        actual: String,
    },
    
    /// Lock acquisition failed
    #[error("Failed to acquire migration lock: {0}")]
    LockError(String),
    
    /// Migration is in a failed state
    #[error("Migration {0} is in a failed state and must be resolved manually")]
    FailedState(i64),
    
    /// Invalid migration version
    #[error("Invalid migration version: {0}")]
    InvalidVersion(i64),
    
    /// Migration gap detected
    #[error("Migration gap detected: missing version {0}")]
    MigrationGap(i64),
    
    /// IO error occurred
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    /// Custom error
    #[error("{0}")]
    Custom(String),
}

impl MigrationError {
    /// Create a new database error
    pub fn database<S: Into<String>>(msg: S) -> Self {
        Self::DatabaseError(msg.into())
    }
    
    /// Create a new custom error
    pub fn custom<S: Into<String>>(msg: S) -> Self {
        Self::Custom(msg.into())
    }
}

/// Result type alias for migration operations
pub type Result<T> = std::result::Result<T, MigrationError>;

// Database-specific error conversions
// Note: postgres::Error and tokio_postgres::Error are the same type,
// so we only need one implementation
#[cfg(any(feature = "postgres", feature = "tokio-postgres"))]
impl From<postgres::Error> for MigrationError {
    fn from(err: postgres::Error) -> Self {
        Self::DatabaseError(err.to_string())
    }
}

#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for MigrationError {
    fn from(err: rusqlite::Error) -> Self {
        Self::DatabaseError(err.to_string())
    }
}

#[cfg(feature = "deadpool-postgres")]
impl From<deadpool_postgres::PoolError> for MigrationError {
    fn from(err: deadpool_postgres::PoolError) -> Self {
        Self::DatabaseError(format!("Connection pool error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = MigrationError::AlreadyApplied(1);
        assert_eq!(err.to_string(), "Migration 1 has already been applied");
        
        let err = MigrationError::ChecksumMismatch {
            version: 2,
            expected: "abc123".to_string(),
            actual: "def456".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Checksum mismatch for migration 2: expected abc123, got def456"
        );
    }
    
    #[test]
    fn test_custom_errors() {
        let err = MigrationError::database("connection failed");
        assert_eq!(err.to_string(), "Database error: connection failed");
        
        let err = MigrationError::custom("something went wrong");
        assert_eq!(err.to_string(), "something went wrong");
    }
}