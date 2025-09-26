//! Tests for the migration runner.

mod common;

use common::*;
use parsql_migrations::{
    prelude::*,
    traits_simple::MigrationRecord,
};

#[test]
fn test_run_migrations_success() {
    let mut conn = TestConnection::new();
    let mut runner = MigrationRunner::new();
    
    // Add test migrations
    let migrations = create_test_migrations();
    for migration in migrations {
        runner.add_migration(migration);
    }
    
    // Run migrations
    let report = runner.run(&mut conn).unwrap();
    
    // Verify results
    assert_eq!(report.successful_count(), 3);
    assert_eq!(report.failed_count(), 0);
    assert!(report.is_success());
    
    // Check executed queries
    let queries = conn.get_executed_queries();
    assert!(queries.iter().any(|q| q.contains("CREATE TABLE")));
    assert!(queries.iter().any(|q| q.contains("parsql_migrations")));
}

#[test]
fn test_skip_already_applied_migrations() {
    let mut conn = TestConnection::new();
    
    // Pre-populate with an applied migration
    conn.add_migration(MigrationRecord {
        version: 1,
        name: "create_users_table".to_string(),
        applied_at: chrono::Utc::now(),
        checksum: Some("test_checksum".to_string()),
        execution_time_ms: Some(50),
    });
    
    // Create a runner that allows out-of-order execution
    let mut runner = MigrationRunner::with_config(
        MigrationConfig::new().allow_out_of_order(true)
    );
    let migrations = create_test_migrations();
    for migration in migrations {
        runner.add_migration(migration);
    }
    
    // Run migrations
    let report = runner.run(&mut conn).unwrap();
    
    // Migration 1 should be skipped
    assert_eq!(report.successful_count(), 2); // Only 2 and 3 should run
    assert_eq!(report.skipped.len(), 1);
    assert_eq!(report.skipped[0], 1);
}

#[test]
fn test_migration_failure_stops_execution() {
    let mut conn = TestConnection::new().with_migration_failure_only();
    let config = MigrationConfig::new()
        .with_auto_create_table(false)
        .with_transactions(false);
    let mut runner = MigrationRunner::with_config(config);
    
    let migrations = create_test_migrations();
    for migration in migrations {
        runner.add_migration(migration);
    }
    
    // Run migrations - should return a report with failures
    let report = runner.run(&mut conn).expect("Runner should succeed even when migrations fail");
    
    // First migration should fail, and execution should stop
    assert_eq!(report.successful_count(), 0);
    assert_eq!(report.failed_count(), 1);
    assert!(!report.is_success());
}

#[test]
fn test_migration_with_transaction() {
    let mut conn = TestConnection::new();
    let mut runner = MigrationRunner::with_config(
        MigrationConfig::new().with_transactions(true)
    );
    
    runner.add_migration(Box::new(TestMigration::new(
        1,
        "test_migration",
        "CREATE TABLE test (id INT)",
        "DROP TABLE test"
    )));
    
    let report = runner.run(&mut conn).unwrap();
    assert!(report.is_success());
    
    // Verify transaction commands were used
    let queries = conn.get_executed_queries();
    assert!(queries.iter().any(|q| q == "BEGIN"));
    assert!(queries.iter().any(|q| q == "COMMIT"));
}

#[test]
fn test_migration_rollback() {
    let mut conn = TestConnection::new();
    let mut runner = MigrationRunner::new();
    
    // Add and run migrations
    let migrations = create_test_migrations();
    for migration in migrations {
        runner.add_migration(migration);
    }
    
    let run_report = runner.run(&mut conn).unwrap();
    assert_eq!(run_report.successful_count(), 3);
    
    // Clear executed queries for rollback test
    conn.clear();
    
    // Add applied migrations to simulate they were run
    for i in 1..=3 {
        conn.add_migration(MigrationRecord {
            version: i,
            name: format!("migration_{}", i),
            applied_at: chrono::Utc::now(),
            checksum: None,
            execution_time_ms: Some(10),
        });
    }
    
    // Rollback to version 1 (should rollback 3 and 2)
    let rollback_report = runner.rollback(&mut conn, 1).unwrap();
    
    assert_eq!(rollback_report.successful_count(), 2);
    let queries = conn.get_executed_queries();
    assert!(queries.iter().any(|q| q.contains("DROP TABLE posts")));
    assert!(queries.iter().any(|q| q.contains("ALTER TABLE users DROP COLUMN email")));
}

#[test]
fn test_migration_status() {
    let mut conn = TestConnection::new();
    let mut runner = MigrationRunner::new();
    
    // Add migrations
    let migrations = create_test_migrations();
    for migration in migrations {
        runner.add_migration(migration);
    }
    
    // Apply first migration only
    conn.add_migration(MigrationRecord {
        version: 1,
        name: "create_users_table".to_string(),
        applied_at: chrono::Utc::now(),
        checksum: Some("checksum1".to_string()),
        execution_time_ms: Some(15),
    });
    
    // Get status
    let status = runner.status(&mut conn).unwrap();
    
    assert_eq!(status.len(), 3);
    
    // First migration should be applied
    assert_eq!(status[0].version, 1);
    assert!(status[0].applied);
    assert!(status[0].applied_at.is_some());
    
    // Others should not be applied
    assert_eq!(status[1].version, 2);
    assert!(!status[1].applied);
    assert!(status[1].applied_at.is_none());
    
    assert_eq!(status[2].version, 3);
    assert!(!status[2].applied);
}

#[test]
fn test_config_options() {
    let config = MigrationConfig::new()
        .with_table_name("custom_migrations")
        .with_transactions(false)
        .with_checksum_verification(false)
        .stop_on_error(false);
    
    assert_eq!(config.table.table_name, "custom_migrations");
    assert!(!config.transaction_per_migration);
    assert!(!config.verify_checksums);
    assert!(!config.stop_on_error);
}

#[test]
fn test_migration_gap_detection() {
    let mut conn = TestConnection::new();
    let config = MigrationConfig::new()
        .allow_out_of_order(false); // Don't allow out-of-order
    
    let mut runner = MigrationRunner::with_config(config);
    
    // Add migrations 1 and 3 (skip 2)
    runner.add_migration(Box::new(TestMigration::new(
        1,
        "migration_1",
        "CREATE TABLE t1 (id INT)",
        "DROP TABLE t1"
    )));
    
    runner.add_migration(Box::new(TestMigration::new(
        3,
        "migration_3",
        "CREATE TABLE t3 (id INT)",
        "DROP TABLE t3"
    )));
    
    // Apply migration 1
    conn.add_migration(MigrationRecord {
        version: 1,
        name: "migration_1".to_string(),
        applied_at: chrono::Utc::now(),
        checksum: None,
        execution_time_ms: Some(10),
    });
    
    // Running should succeed - migration 3 can run after 1
    let report = runner.run(&mut conn).unwrap();
    assert_eq!(report.successful_count(), 1); // Only migration 3
}