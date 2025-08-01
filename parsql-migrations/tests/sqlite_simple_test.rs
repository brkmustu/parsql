//! SQLite integration tests for the migration system.

#[cfg(feature = "sqlite")]
mod sqlite_tests {
    use parsql_migrations::{
        prelude::*,
        sqlite_simple::SqliteConnectionExt,
        traits_simple::{Migration, MigrationConnection},
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
                    email TEXT UNIQUE
                )"
            )
        }
        
        fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
            conn.execute("DROP TABLE users")
        }
    }

    struct AddCreatedAt;

    impl Migration for AddCreatedAt {
        fn version(&self) -> i64 { 2 }
        fn name(&self) -> &str { "add_created_at" }
        
        fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
            conn.execute("ALTER TABLE users ADD COLUMN created_at TEXT DEFAULT CURRENT_TIMESTAMP")
        }
        
        fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
            // SQLite doesn't support DROP COLUMN directly
            conn.execute(
                "CREATE TABLE users_new AS SELECT id, name, email FROM users;
                 DROP TABLE users;
                 ALTER TABLE users_new RENAME TO users;"
            )
        }
    }

    #[test]
    fn test_sqlite_migration_runner() {
        // Create a temporary database file
        let temp_file = NamedTempFile::new().unwrap();
        let mut conn = Connection::open(temp_file.path()).unwrap();
        
        // Create migration runner
        let mut runner = MigrationRunner::new();
        runner.add_migration(Box::new(CreateUsersTable));
        runner.add_migration(Box::new(AddCreatedAt));
        
        // Get migration connection
        let mut migration_conn = conn.migration_connection();
        
        // Run migrations
        let report = runner.run(&mut migration_conn).unwrap();
        
        // Verify results
        assert_eq!(report.successful_count(), 2);
        assert_eq!(report.failed_count(), 0);
        assert!(report.is_success());
        
        // Verify tables exist
        let table_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'",
                [],
                |row| row.get(0)
            )
            .unwrap();
        assert_eq!(table_exists, 1);
        
        // Verify columns exist
        let mut stmt = conn.prepare("PRAGMA table_info(users)").unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        
        assert!(columns.contains(&"id".to_string()));
        assert!(columns.contains(&"name".to_string()));
        assert!(columns.contains(&"email".to_string()));
        assert!(columns.contains(&"created_at".to_string()));
    }

    #[test]
    fn test_sqlite_migration_status() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut conn = Connection::open(temp_file.path()).unwrap();
        
        let mut runner = MigrationRunner::new();
        runner.add_migration(Box::new(CreateUsersTable));
        runner.add_migration(Box::new(AddCreatedAt));
        
        let mut migration_conn = conn.migration_connection();
        
        // Run first migration only
        let mut runner_single = MigrationRunner::new();
        runner_single.add_migration(Box::new(CreateUsersTable));
        let _ = runner_single.run(&mut migration_conn).unwrap();
        
        // Check status
        let status = runner.status(&mut migration_conn).unwrap();
        
        assert_eq!(status.len(), 2);
        assert!(status[0].applied);
        assert!(!status[1].applied);
    }

    #[test]
    fn test_sqlite_migration_rollback() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut conn = Connection::open(temp_file.path()).unwrap();
        
        let mut runner = MigrationRunner::new();
        runner.add_migration(Box::new(CreateUsersTable));
        runner.add_migration(Box::new(AddCreatedAt));
        
        let mut migration_conn = conn.migration_connection();
        
        // Run all migrations
        let _ = runner.run(&mut migration_conn).unwrap();
        
        // Rollback to version 0 (all migrations)
        let rollback_report = runner.rollback(&mut migration_conn, 0).unwrap();
        
        assert_eq!(rollback_report.successful_count(), 2);
        assert!(rollback_report.is_success());
        
        // Verify table doesn't exist
        let table_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'",
                [],
                |row| row.get(0)
            )
            .unwrap();
        assert_eq!(table_exists, 0);
    }

    #[test]
    fn test_sqlite_migration_with_transactions() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut conn = Connection::open(temp_file.path()).unwrap();
        
        struct FailingMigration;
        
        impl Migration for FailingMigration {
            fn version(&self) -> i64 { 3 }
            fn name(&self) -> &str { "failing_migration" }
            
            fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
                conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY)")?;
                // This will fail
                conn.execute("INVALID SQL STATEMENT")
            }
            
            fn down(&self, _conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
                Ok(())
            }
        }
        
        let config = MigrationConfig::new().with_transactions(true);
        let mut runner = MigrationRunner::with_config(config);
        runner.add_migration(Box::new(FailingMigration));
        
        let mut migration_conn = conn.migration_connection();
        let report = runner.run(&mut migration_conn).unwrap();
        
        // Migration should fail
        assert_eq!(report.failed_count(), 1);
        
        // Table should not exist due to transaction rollback
        let table_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='test'",
                [],
                |row| row.get(0)
            )
            .unwrap();
        assert_eq!(table_exists, 0);
    }
}