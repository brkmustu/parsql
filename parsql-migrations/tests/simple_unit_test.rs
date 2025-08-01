//! Simple unit tests for the migration system that don't rely on database connections.

use parsql_migrations::types::{MigrationReport, MigrationResult, MigrationStatus};

#[test]
fn test_migration_report() {
    let mut report = MigrationReport::new();
    
    // Add some successful migrations
    report.add_success(MigrationResult::success(1, "create_users".to_string(), 100));
    report.add_success(MigrationResult::success(2, "add_email".to_string(), 50));
    
    // Add a failed migration
    report.add_failure(MigrationResult::failure(3, "create_posts".to_string(), "connection error".to_string(), 10));
    
    // Add a skipped migration
    report.add_skipped(4);
    
    // Test assertions before completion
    assert_eq!(report.successful_count(), 2);
    assert_eq!(report.failed_count(), 1);
    assert_eq!(report.skipped.len(), 1);
    assert!(!report.is_success()); // Should be false because of failure
    
    // Complete the report
    report.complete();
    
    // Test assertions after completion
    assert!(report.total_time_ms >= 0); // Should be non-negative
    assert!(report.completed_at.is_some());
}

#[test]
fn test_migration_status() {
    let status_applied = MigrationStatus {
        version: 1,
        name: "create_users".to_string(),
        applied: true,
        applied_at: Some(chrono::Utc::now()),
        execution_time_ms: Some(100),
    };
    
    let status_pending = MigrationStatus {
        version: 2,
        name: "add_email".to_string(),
        applied: false,
        applied_at: None,
        execution_time_ms: None,
    };
    
    assert!(status_applied.applied);
    assert!(!status_pending.applied);
    assert!(status_applied.applied_at.is_some());
    assert!(status_pending.applied_at.is_none());
}

#[test]
fn test_migration_result_success() {
    let result = MigrationResult::success(1, "test_migration".to_string(), 50);
    
    assert_eq!(result.version, 1);
    assert_eq!(result.name, "test_migration");
    assert!(result.success);
    assert!(result.error.is_none());
    assert_eq!(result.execution_time_ms, 50);
}

#[test]
fn test_migration_result_failure() {
    let result = MigrationResult::failure(2, "failed_migration".to_string(), "database error".to_string(), 10);
    
    assert_eq!(result.version, 2);
    assert_eq!(result.name, "failed_migration");
    assert!(!result.success);
    assert_eq!(result.error, Some("database error".to_string()));
    assert_eq!(result.execution_time_ms, 10);
}