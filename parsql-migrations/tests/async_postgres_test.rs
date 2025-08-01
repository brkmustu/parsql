//! Async PostgreSQL integration tests.

#![cfg(feature = "tokio-postgres")]

use parsql_migrations::{
    prelude::*,
    traits_v2::{AsyncMigration, AsyncMigrationConnection},
    tokio_postgres::{TokioPostgresMigrationConnection, AsyncMigrationRunner},
};
use tokio_postgres::{NoTls};
use async_trait::async_trait;
use std::env;

struct CreateUsersTableAsync;

#[async_trait]
impl AsyncMigration for CreateUsersTableAsync {
    fn version(&self) -> i64 { 1 }
    fn name(&self) -> &str { "create_users_table" }
    
    async fn up(&self, conn: &mut dyn AsyncMigrationConnection) -> Result<(), MigrationError> {
        conn.execute(
            "CREATE TABLE users (
                id BIGSERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) UNIQUE NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )"
        ).await
    }
    
    async fn down(&self, conn: &mut dyn AsyncMigrationConnection) -> Result<(), MigrationError> {
        conn.execute("DROP TABLE IF EXISTS users").await
    }
}

struct AddUserStatusAsync;

#[async_trait]
impl AsyncMigration for AddUserStatusAsync {
    fn version(&self) -> i64 { 2 }
    fn name(&self) -> &str { "add_user_status" }
    
    async fn up(&self, conn: &mut dyn AsyncMigrationConnection) -> Result<(), MigrationError> {
        conn.execute("ALTER TABLE users ADD COLUMN status VARCHAR(50) DEFAULT 'active'").await
    }
    
    async fn down(&self, conn: &mut dyn AsyncMigrationConnection) -> Result<(), MigrationError> {
        conn.execute("ALTER TABLE users DROP COLUMN status").await
    }
}

fn get_test_db_url() -> String {
    env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "host=localhost user=postgres password=postgres dbname=parsql_test".to_string()
    })
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_async_postgres_migrations() {
    let (client, connection) = tokio_postgres::connect(&get_test_db_url(), NoTls)
        .await
        .expect("Failed to connect to test database");
    
    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    // Clean up
    let _ = client.execute("DROP TABLE IF EXISTS users CASCADE", &[]).await;
    let _ = client.execute("DROP TABLE IF EXISTS parsql_migrations", &[]).await;
    
    let mut migration_conn = TokioPostgresMigrationConnection::new(&client);
    
    let mut runner = AsyncMigrationRunner::new();
    runner.add_migration(Box::new(CreateUsersTableAsync));
    runner.add_migration(Box::new(AddUserStatusAsync));
    
    // Run migrations
    let report = runner.run(&mut migration_conn).await.unwrap();
    
    assert_eq!(report.successful_count(), 2);
    assert_eq!(report.failed_count(), 0);
    
    // Verify table structure
    let columns: Vec<String> = client
        .query(
            "SELECT column_name FROM information_schema.columns 
             WHERE table_name = 'users' 
             ORDER BY ordinal_position",
            &[],
        )
        .await
        .unwrap()
        .iter()
        .map(|row| row.get::<_, String>(0))
        .collect();
    
    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"name".to_string()));
    assert!(columns.contains(&"email".to_string()));
    assert!(columns.contains(&"status".to_string()));
    assert!(columns.contains(&"created_at".to_string()));
}

#[tokio::test]
#[ignore]
async fn test_async_concurrent_migrations() {
    // Test that multiple migration runners don't interfere with each other
    let (client1, connection1) = tokio_postgres::connect(&get_test_db_url(), NoTls)
        .await
        .expect("Failed to connect to test database");
    
    let (client2, connection2) = tokio_postgres::connect(&get_test_db_url(), NoTls)
        .await
        .expect("Failed to connect to test database");
    
    tokio::spawn(async move {
        if let Err(e) = connection1.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    tokio::spawn(async move {
        if let Err(e) = connection2.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    // Clean up
    let _ = client1.execute("DROP TABLE IF EXISTS users CASCADE", &[]).await;
    let _ = client1.execute("DROP TABLE IF EXISTS parsql_migrations", &[]).await;
    
    let mut conn1 = TokioPostgresMigrationConnection::new(&client1);
    let mut conn2 = TokioPostgresMigrationConnection::new(&client2);
    
    let mut runner1 = AsyncMigrationRunner::new();
    runner1.add_migration(Box::new(CreateUsersTableAsync));
    
    let mut runner2 = AsyncMigrationRunner::new();
    runner2.add_migration(Box::new(CreateUsersTableAsync));
    
    // Run migrations concurrently
    let (result1, result2) = tokio::join!(
        runner1.run(&mut conn1),
        runner2.run(&mut conn2)
    );
    
    // One should succeed, one should skip (already applied)
    let report1 = result1.unwrap();
    let report2 = result2.unwrap();
    
    let total_successful = report1.successful_count() + report2.successful_count();
    let total_skipped = report1.skipped.len() + report2.skipped.len();
    
    assert_eq!(total_successful, 1);
    assert_eq!(total_skipped, 1);
}

#[tokio::test]
#[ignore]
async fn test_async_transaction_isolation() {
    let (client, connection) = tokio_postgres::connect(&get_test_db_url(), NoTls)
        .await
        .expect("Failed to connect to test database");
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    // Clean up
    let _ = client.execute("DROP TABLE IF EXISTS test_isolation CASCADE", &[]).await;
    let _ = client.execute("DROP TABLE IF EXISTS parsql_migrations", &[]).await;
    
    struct FailingAsyncMigration;
    
    #[async_trait]
    impl AsyncMigration for FailingAsyncMigration {
        fn version(&self) -> i64 { 3 }
        fn name(&self) -> &str { "failing_migration" }
        
        async fn up(&self, conn: &mut dyn AsyncMigrationConnection) -> Result<(), MigrationError> {
            conn.execute("CREATE TABLE test_isolation (id SERIAL PRIMARY KEY)").await?;
            conn.execute("INSERT INTO test_isolation (id) VALUES (1)").await?;
            // Force a failure
            Err(MigrationError::Custom("Simulated failure".into()))
        }
        
        async fn down(&self, _conn: &mut dyn AsyncMigrationConnection) -> Result<(), MigrationError> {
            Ok(())
        }
    }
    
    let mut migration_conn = TokioPostgresMigrationConnection::new(&client);
    let mut runner = AsyncMigrationRunner::new();
    runner.add_migration(Box::new(FailingAsyncMigration));
    
    let report = runner.run(&mut migration_conn).await.unwrap();
    assert_eq!(report.failed_count(), 1);
    
    // Verify the table was NOT created (transaction rolled back)
    let table_exists = client
        .query_one(
            "SELECT COUNT(*) FROM information_schema.tables 
             WHERE table_name = 'test_isolation'",
            &[],
        )
        .await
        .unwrap()
        .get::<_, i64>(0);
    
    assert_eq!(table_exists, 0);
}