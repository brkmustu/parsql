//! Common types used throughout the migration system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status of a migration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationState {
    /// Migration has been successfully applied
    Applied,
    /// Migration failed during execution
    Failed,
    /// Migration is currently being applied
    InProgress,
    /// Migration has been rolled back
    RolledBack,
}

/// Detailed information about a single migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationDetails {
    /// Unique version identifier
    pub version: i64,
    /// Human-readable name
    pub name: String,
    /// Current state of the migration
    pub state: MigrationState,
    /// When the migration was applied
    pub applied_at: Option<DateTime<Utc>>,
    /// When the migration was rolled back (if applicable)
    pub rolled_back_at: Option<DateTime<Utc>>,
    /// Checksum for verification
    pub checksum: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: Option<i64>,
    /// Error message if migration failed
    pub error_message: Option<String>,
}

impl MigrationDetails {
    /// Create a new migration details instance
    pub fn new(version: i64, name: String) -> Self {
        Self {
            version,
            name,
            state: MigrationState::InProgress,
            applied_at: None,
            rolled_back_at: None,
            checksum: None,
            execution_time_ms: None,
            error_message: None,
        }
    }
    
    /// Mark migration as successfully applied
    pub fn mark_applied(&mut self, execution_time_ms: i64) {
        self.state = MigrationState::Applied;
        self.applied_at = Some(Utc::now());
        self.execution_time_ms = Some(execution_time_ms);
        self.error_message = None;
    }
    
    /// Mark migration as failed
    pub fn mark_failed(&mut self, error: String) {
        self.state = MigrationState::Failed;
        self.error_message = Some(error);
    }
    
    /// Mark migration as rolled back
    pub fn mark_rolled_back(&mut self) {
        self.state = MigrationState::RolledBack;
        self.rolled_back_at = Some(Utc::now());
    }
}

/// Status information for a migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    /// Migration version
    pub version: i64,
    /// Migration name
    pub name: String,
    /// Whether the migration has been applied
    pub applied: bool,
    /// When the migration was applied
    pub applied_at: Option<DateTime<Utc>>,
    /// Execution time in milliseconds
    pub execution_time_ms: Option<i64>,
}

/// Report of migration operations
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MigrationReport {
    /// Successfully applied migrations
    pub successful: Vec<MigrationResult>,
    /// Failed migrations
    pub failed: Vec<MigrationResult>,
    /// Skipped migrations (already applied)
    pub skipped: Vec<i64>,
    /// Total execution time in milliseconds
    pub total_time_ms: i64,
    /// Start time of the operation
    pub started_at: DateTime<Utc>,
    /// End time of the operation
    pub completed_at: Option<DateTime<Utc>>,
}

impl MigrationReport {
    /// Create a new migration report
    pub fn new() -> Self {
        Self {
            started_at: Utc::now(),
            ..Default::default()
        }
    }
    
    /// Add a successful migration result
    pub fn add_success(&mut self, result: MigrationResult) {
        self.successful.push(result);
    }
    
    /// Add a failed migration result
    pub fn add_failure(&mut self, result: MigrationResult) {
        self.failed.push(result);
    }
    
    /// Add a skipped migration
    pub fn add_skipped(&mut self, version: i64) {
        self.skipped.push(version);
    }
    
    /// Mark the report as completed
    pub fn complete(&mut self) {
        self.completed_at = Some(Utc::now());
        if let Some(completed) = self.completed_at {
            self.total_time_ms = (completed - self.started_at).num_milliseconds();
        }
    }
    
    /// Get the number of successful migrations
    pub fn successful_count(&self) -> usize {
        self.successful.len()
    }
    
    /// Get the number of failed migrations
    pub fn failed_count(&self) -> usize {
        self.failed.len()
    }
    
    /// Check if all migrations were successful
    pub fn is_success(&self) -> bool {
        self.failed.is_empty()
    }
    
    /// Get a summary of the report
    pub fn summary(&self) -> String {
        format!(
            "Migration Report: {} successful, {} failed, {} skipped ({}ms)",
            self.successful_count(),
            self.failed_count(),
            self.skipped.len(),
            self.total_time_ms
        )
    }
}

/// Result of a single migration operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    /// Migration version
    pub version: i64,
    /// Migration name
    pub name: String,
    /// Whether the operation was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: i64,
    /// Timestamp of execution
    pub executed_at: DateTime<Utc>,
}

impl MigrationResult {
    /// Create a successful migration result
    pub fn success(version: i64, name: String, execution_time_ms: i64) -> Self {
        Self {
            version,
            name,
            success: true,
            error: None,
            execution_time_ms,
            executed_at: Utc::now(),
        }
    }
    
    /// Create a failed migration result
    pub fn failure(version: i64, name: String, error: String, execution_time_ms: i64) -> Self {
        Self {
            version,
            name,
            success: false,
            error: Some(error),
            execution_time_ms,
            executed_at: Utc::now(),
        }
    }
}

/// Configuration for table names and columns
#[derive(Debug, Clone)]
pub struct TableConfig {
    /// Name of the migrations table
    pub table_name: String,
    /// Name of the version column
    pub version_column: String,
    /// Name of the name column
    pub name_column: String,
    /// Name of the applied_at column
    pub applied_at_column: String,
    /// Name of the checksum column
    pub checksum_column: String,
    /// Name of the execution_time_ms column
    pub execution_time_column: String,
    /// Name of the rolled_back_at column
    pub rolled_back_at_column: String,
}

impl Default for TableConfig {
    fn default() -> Self {
        Self {
            table_name: "parsql_migrations".to_string(),
            version_column: "version".to_string(),
            name_column: "name".to_string(),
            applied_at_column: "applied_at".to_string(),
            checksum_column: "checksum".to_string(),
            execution_time_column: "execution_time_ms".to_string(),
            rolled_back_at_column: "rolled_back_at".to_string(),
        }
    }
}

/// Type alias for migration version to details mapping
pub type MigrationMap = HashMap<i64, MigrationDetails>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_migration_report() {
        let mut report = MigrationReport::new();
        
        report.add_success(MigrationResult::success(1, "test1".to_string(), 100));
        report.add_failure(MigrationResult::failure(
            2, 
            "test2".to_string(), 
            "error".to_string(), 
            50
        ));
        report.add_skipped(3);
        
        assert_eq!(report.successful_count(), 1);
        assert_eq!(report.failed_count(), 1);
        assert!(!report.is_success());
        
        report.complete();
        assert!(report.completed_at.is_some());
    }
    
    #[test]
    fn test_migration_details() {
        let mut details = MigrationDetails::new(1, "test".to_string());
        assert_eq!(details.state, MigrationState::InProgress);
        
        details.mark_applied(100);
        assert_eq!(details.state, MigrationState::Applied);
        assert!(details.applied_at.is_some());
        assert_eq!(details.execution_time_ms, Some(100));
        
        details.mark_rolled_back();
        assert_eq!(details.state, MigrationState::RolledBack);
        assert!(details.rolled_back_at.is_some());
    }
}