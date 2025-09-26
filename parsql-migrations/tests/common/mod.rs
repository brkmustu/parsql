//! Common test utilities for migration tests.

use parsql_migrations::{
    prelude::*,
    traits_simple::{Migration, MigrationConnection, MigrationRecord},
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Test implementation of a migration
pub struct TestMigration {
    version: i64,
    name: String,
    up_sql: String,
    down_sql: String,
}

impl TestMigration {
    pub fn new(version: i64, name: impl Into<String>, up_sql: impl Into<String>, down_sql: impl Into<String>) -> Self {
        Self {
            version,
            name: name.into(),
            up_sql: up_sql.into(),
            down_sql: down_sql.into(),
        }
    }
}

impl Migration for TestMigration {
    fn version(&self) -> i64 {
        self.version
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute(&self.up_sql)
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute(&self.down_sql)
    }
}

/// In-memory test database connection
pub struct TestConnection {
    executed_queries: Arc<Mutex<Vec<String>>>,
    migrations: Arc<Mutex<HashMap<i64, MigrationRecord>>>,
    in_transaction: bool,
    should_fail: bool,
    fail_migrations_only: bool, // New field for selective failure
}

impl TestConnection {
    pub fn new() -> Self {
        Self {
            executed_queries: Arc::new(Mutex::new(Vec::new())),
            migrations: Arc::new(Mutex::new(HashMap::new())),
            in_transaction: false,
            should_fail: false,
            fail_migrations_only: false,
        }
    }
    
    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
    
    pub fn with_migration_failure_only(mut self) -> Self {
        self.fail_migrations_only = true;
        self
    }
    
    pub fn get_executed_queries(&self) -> Vec<String> {
        self.executed_queries.lock().unwrap().clone()
    }
    
    pub fn add_migration(&self, record: MigrationRecord) {
        self.migrations.lock().unwrap().insert(record.version, record);
    }
    
    pub fn clear(&self) {
        self.executed_queries.lock().unwrap().clear();
        self.migrations.lock().unwrap().clear();
    }
}

impl MigrationConnection for TestConnection {
    fn execute(&mut self, sql: &str) -> Result<(), MigrationError> {
        if self.should_fail {
            return Err(MigrationError::database("Test failure"));
        }
        
        // Fail only on migration SQL (CREATE TABLE, ALTER TABLE, DROP TABLE)
        if self.fail_migrations_only && (sql.contains("CREATE TABLE") || sql.contains("ALTER TABLE") || sql.contains("DROP TABLE")) {
            return Err(MigrationError::database("Test migration failure"));
        }
        
        self.executed_queries.lock().unwrap().push(sql.to_string());
        
        // Simulate migration table operations
        if sql.contains("INSERT INTO") && sql.contains("parsql_migrations") {
            // Parse version from SQL (simplified)
            if let Some(version_str) = sql.split('(').nth(2).and_then(|s| s.split(',').next()) {
                if let Ok(version) = version_str.trim().parse::<i64>() {
                    let record = MigrationRecord {
                        version,
                        name: format!("migration_{}", version),
                        applied_at: chrono::Utc::now(),
                        checksum: Some(format!("checksum_{}", version)),
                        execution_time_ms: Some(100),
                    };
                    self.migrations.lock().unwrap().insert(version, record);
                }
            }
        }
        
        Ok(())
    }
    
    fn begin_transaction(&mut self) -> Result<(), MigrationError> {
        self.in_transaction = true;
        self.executed_queries.lock().unwrap().push("BEGIN".to_string());
        Ok(())
    }
    
    fn commit_transaction(&mut self) -> Result<(), MigrationError> {
        self.executed_queries.lock().unwrap().push("COMMIT".to_string());
        self.in_transaction = false;
        Ok(())
    }
    
    fn rollback_transaction(&mut self) -> Result<(), MigrationError> {
        self.executed_queries.lock().unwrap().push("ROLLBACK".to_string());
        self.in_transaction = false;
        Ok(())
    }
    
    fn database_type(&self) -> &str {
        "test"
    }
    
    fn query_migrations(&mut self, _table_name: &str) -> Result<Vec<MigrationRecord>, MigrationError> {
        if self.should_fail {
            return Err(MigrationError::database("Test failure"));
        }
        Ok(self.migrations.lock().unwrap().values().cloned().collect())
    }
}

/// Create a set of test migrations
pub fn create_test_migrations() -> Vec<Box<dyn Migration>> {
    vec![
        Box::new(TestMigration::new(
            1,
            "create_users_table",
            "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(255))",
            "DROP TABLE users"
        )),
        Box::new(TestMigration::new(
            2,
            "add_email_to_users",
            "ALTER TABLE users ADD COLUMN email VARCHAR(255)",
            "ALTER TABLE users DROP COLUMN email"
        )),
        Box::new(TestMigration::new(
            3,
            "create_posts_table",
            "CREATE TABLE posts (id INT PRIMARY KEY, user_id INT, title VARCHAR(255))",
            "DROP TABLE posts"
        )),
    ]
}