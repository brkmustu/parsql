//! SQLite integration tests for migrations.

#![cfg(feature = "sqlite")]

use parsql_migrations::{
    prelude::*,
    traits_v2::{Migration, MigrationConnection, MigrationRecord},
    sqlite::SqliteMigrationConnection,
};
use rusqlite::Connection;
use tempfile::NamedTempFile;

struct CreateUsersTable;

impl Migration for CreateUsersTable {
    fn version(&self) -> i64 { 1 }
    fn name(&self) -> &str { "create_users_table" }
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT UNIQUE NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )"
        )
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute("DROP TABLE IF EXISTS users")
    }
}

struct AddUserIndex;

impl Migration for AddUserIndex {
    fn version(&self) -> i64 { 2 }
    fn name(&self) -> &str { "add_user_email_index" }
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute("CREATE INDEX idx_users_email ON users(email)")
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute("DROP INDEX IF EXISTS idx_users_email")
    }
}

#[test]
fn test_sqlite_migrations() {
    // Create a temporary SQLite database
    let temp_file = NamedTempFile::new().unwrap();
    let mut conn = Connection::open(temp_file.path()).unwrap();
    
    // Create migration connection
    let mut migration_conn = SqliteMigrationConnection::new(&mut conn);
    
    // Create and run migrations
    let mut runner = MigrationRunner::new();
    runner.add_migration(Box::new(CreateUsersTable));
    runner.add_migration(Box::new(AddUserIndex));
    
    // Run migrations
    let report = runner.run(&mut migration_conn).unwrap();
    
    assert_eq!(report.successful_count(), 2);
    assert_eq!(report.failed_count(), 0);
    
    // Verify the tables were created
    let table_exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    
    assert_eq!(table_exists, 1);
    
    // Verify the index was created
    let index_exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_users_email'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    
    assert_eq!(index_exists, 1);
    
    // Test migration status
    let status = runner.status(&mut migration_conn).unwrap();
    assert_eq!(status.len(), 2);
    assert!(status[0].applied);
    assert!(status[1].applied);
}

#[test]
fn test_sqlite_rollback() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut conn = Connection::open(temp_file.path()).unwrap();
    let mut migration_conn = SqliteMigrationConnection::new(&mut conn);
    
    let mut runner = MigrationRunner::new();
    runner.add_migration(Box::new(CreateUsersTable));
    runner.add_migration(Box::new(AddUserIndex));
    
    // Run migrations
    runner.run(&mut migration_conn).unwrap();
    
    // Rollback to version 0 (rollback everything)
    let rollback_report = runner.rollback(&mut migration_conn, 0).unwrap();
    
    assert_eq!(rollback_report.successful_count(), 2);
    
    // Verify the table was dropped
    let table_exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    
    assert_eq!(table_exists, 0);
}

#[test]
fn test_sqlite_transaction_rollback_on_error() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut conn = Connection::open(temp_file.path()).unwrap();
    let mut migration_conn = SqliteMigrationConnection::new(&mut conn);
    
    struct FailingMigration;
    
    impl Migration for FailingMigration {
        fn version(&self) -> i64 { 3 }
        fn name(&self) -> &str { "failing_migration" }
        
        fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
            conn.execute("CREATE TABLE test_table (id INTEGER PRIMARY KEY)")?;
            // This will fail because table already exists
            conn.execute("CREATE TABLE test_table (id INTEGER PRIMARY KEY)")
        }
        
        fn down(&self, _conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
            Ok(())
        }
    }
    
    let mut runner = MigrationRunner::new();
    runner.add_migration(Box::new(FailingMigration));
    
    // Run migration - should fail
    let report = runner.run(&mut migration_conn).unwrap();
    
    assert_eq!(report.failed_count(), 1);
    
    // Verify the table was NOT created (transaction rolled back)
    let table_exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='test_table'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    
    assert_eq!(table_exists, 0);
}