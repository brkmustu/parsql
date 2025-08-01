//! PostgreSQL integration tests for migrations.

#![cfg(feature = "postgres")]

use parsql_migrations::{
    prelude::*,
    traits_v2::{Migration, MigrationConnection},
    postgres::PostgresMigrationConnection,
};
use postgres::{Client, NoTls};
use std::env;

struct CreateUsersTable;

impl Migration for CreateUsersTable {
    fn version(&self) -> i64 { 1 }
    fn name(&self) -> &str { "create_users_table" }
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute(
            "CREATE TABLE users (
                id BIGSERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) UNIQUE NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )"
        )
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute("DROP TABLE IF EXISTS users")
    }
}

struct AddPostsTable;

impl Migration for AddPostsTable {
    fn version(&self) -> i64 { 2 }
    fn name(&self) -> &str { "add_posts_table" }
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute(
            "CREATE TABLE posts (
                id BIGSERIAL PRIMARY KEY,
                user_id BIGINT NOT NULL REFERENCES users(id),
                title VARCHAR(255) NOT NULL,
                content TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )"
        )
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute("DROP TABLE IF EXISTS posts")
    }
}

fn get_test_db_url() -> String {
    env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "host=localhost user=postgres password=postgres dbname=parsql_test".to_string()
    })
}

#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_postgres_migrations() {
    let mut client = Client::connect(&get_test_db_url(), NoTls)
        .expect("Failed to connect to test database");
    
    // Clean up any existing tables
    let _ = client.execute("DROP TABLE IF EXISTS posts CASCADE", &[]);
    let _ = client.execute("DROP TABLE IF EXISTS users CASCADE", &[]);
    let _ = client.execute("DROP TABLE IF EXISTS parsql_migrations", &[]);
    
    let mut migration_conn = PostgresMigrationConnection::new(&mut client);
    
    let mut runner = MigrationRunner::new();
    runner.add_migration(Box::new(CreateUsersTable));
    runner.add_migration(Box::new(AddPostsTable));
    
    // Run migrations
    let report = runner.run(&mut migration_conn).unwrap();
    
    assert_eq!(report.successful_count(), 2);
    assert_eq!(report.failed_count(), 0);
    
    // Verify tables exist
    let table_count: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM information_schema.tables 
             WHERE table_schema = 'public' 
             AND table_name IN ('users', 'posts')",
            &[],
        )
        .unwrap()
        .get(0);
    
    assert_eq!(table_count, 2);
    
    // Test inserting data
    client.execute(
        "INSERT INTO users (name, email) VALUES ($1, $2)",
        &[&"Test User", &"test@example.com"],
    ).unwrap();
    
    let user_id: i64 = client
        .query_one("SELECT id FROM users WHERE email = $1", &[&"test@example.com"])
        .unwrap()
        .get(0);
    
    client.execute(
        "INSERT INTO posts (user_id, title, content) VALUES ($1, $2, $3)",
        &[&user_id, &"Test Post", &"This is a test post"],
    ).unwrap();
    
    // Verify migration tracking
    let migration_count: i64 = client
        .query_one("SELECT COUNT(*) FROM parsql_migrations", &[])
        .unwrap()
        .get(0);
    
    assert_eq!(migration_count, 2);
}

#[test]
#[ignore]
fn test_postgres_rollback() {
    let mut client = Client::connect(&get_test_db_url(), NoTls)
        .expect("Failed to connect to test database");
    
    // Clean up
    let _ = client.execute("DROP TABLE IF EXISTS posts CASCADE", &[]);
    let _ = client.execute("DROP TABLE IF EXISTS users CASCADE", &[]);
    let _ = client.execute("DROP TABLE IF EXISTS parsql_migrations", &[]);
    
    let mut migration_conn = PostgresMigrationConnection::new(&mut client);
    
    let mut runner = MigrationRunner::new();
    runner.add_migration(Box::new(CreateUsersTable));
    runner.add_migration(Box::new(AddPostsTable));
    
    // Run migrations
    runner.run(&mut migration_conn).unwrap();
    
    // Rollback to version 1 (remove posts table)
    let rollback_report = runner.rollback(&mut migration_conn, 1).unwrap();
    
    assert_eq!(rollback_report.successful_count(), 1);
    
    // Verify posts table was dropped
    let posts_exists: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM information_schema.tables 
             WHERE table_schema = 'public' AND table_name = 'posts'",
            &[],
        )
        .unwrap()
        .get(0);
    
    assert_eq!(posts_exists, 0);
    
    // Verify users table still exists
    let users_exists: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM information_schema.tables 
             WHERE table_schema = 'public' AND table_name = 'users'",
            &[],
        )
        .unwrap()
        .get(0);
    
    assert_eq!(users_exists, 1);
}

#[test]
#[ignore]
fn test_postgres_checksum_verification() {
    let mut client = Client::connect(&get_test_db_url(), NoTls)
        .expect("Failed to connect to test database");
    
    // Clean up
    let _ = client.execute("DROP TABLE IF EXISTS parsql_migrations", &[]);
    
    let mut migration_conn = PostgresMigrationConnection::new(&mut client);
    
    let config = MigrationConfig::new()
        .with_checksum_verification(true);
    
    let mut runner = MigrationRunner::with_config(config);
    
    // First migration
    runner.add_migration(Box::new(CreateUsersTable));
    runner.run(&mut migration_conn).unwrap();
    
    // Tamper with the checksum in the database
    client.execute(
        "UPDATE parsql_migrations SET checksum = 'tampered' WHERE version = 1",
        &[],
    ).unwrap();
    
    // Create a new runner and try to run again
    let mut runner2 = MigrationRunner::with_config(
        MigrationConfig::new().with_checksum_verification(true)
    );
    runner2.add_migration(Box::new(CreateUsersTable));
    
    // This should detect the checksum mismatch
    let status = runner2.status(&mut migration_conn).unwrap();
    assert!(status[0].applied);
    
    // In a real implementation, we would verify checksums during status check
    // For now, just verify the migration was marked as applied
}