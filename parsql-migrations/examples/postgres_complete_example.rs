//! Complete PostgreSQL migration example

use parsql_migrations::prelude::*;
use parsql_migrations::postgres_simple::PostgresMigrationConnection;
use postgres::{Client, NoTls};
use std::env;

// Migration 1: Create users table
struct CreateUsersTable;

impl Migration for CreateUsersTable {
    fn version(&self) -> i64 {
        1
    }
    
    fn name(&self) -> &str {
        "create_users_table"
    }
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute(
            "CREATE TABLE users (
                id BIGSERIAL PRIMARY KEY,
                username VARCHAR(50) NOT NULL UNIQUE,
                email VARCHAR(255) NOT NULL UNIQUE,
                created_at TIMESTAMPTZ DEFAULT NOW()
            )"
        )
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute("DROP TABLE IF EXISTS users CASCADE")
    }
}

// Migration 2: Create posts table
struct CreatePostsTable;

impl Migration for CreatePostsTable {
    fn version(&self) -> i64 {
        2
    }
    
    fn name(&self) -> &str {
        "create_posts_table"
    }
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute(
            "CREATE TABLE posts (
                id BIGSERIAL PRIMARY KEY,
                user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                title VARCHAR(255) NOT NULL,
                content TEXT,
                published BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMPTZ DEFAULT NOW()
            )"
        )
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute("DROP TABLE IF EXISTS posts CASCADE")
    }
}

// Migration 3: Add updated_at triggers
struct AddUpdatedAtTriggers;

impl Migration for AddUpdatedAtTriggers {
    fn version(&self) -> i64 {
        3
    }
    
    fn name(&self) -> &str {
        "add_updated_at_triggers"
    }
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        // Add updated_at columns
        conn.execute("ALTER TABLE users ADD COLUMN updated_at TIMESTAMPTZ DEFAULT NOW()")?;
        conn.execute("ALTER TABLE posts ADD COLUMN updated_at TIMESTAMPTZ DEFAULT NOW()")?;
        
        // Create trigger function
        conn.execute(
            "CREATE OR REPLACE FUNCTION update_updated_at_column()
            RETURNS TRIGGER AS $$
            BEGIN
                NEW.updated_at = NOW();
                RETURN NEW;
            END;
            $$ language 'plpgsql'"
        )?;
        
        // Add triggers
        conn.execute(
            "CREATE TRIGGER update_users_updated_at 
            BEFORE UPDATE ON users 
            FOR EACH ROW 
            EXECUTE FUNCTION update_updated_at_column()"
        )?;
        
        conn.execute(
            "CREATE TRIGGER update_posts_updated_at 
            BEFORE UPDATE ON posts 
            FOR EACH ROW 
            EXECUTE FUNCTION update_updated_at_column()"
        )?;
        
        Ok(())
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute("DROP TRIGGER IF EXISTS update_users_updated_at ON users")?;
        conn.execute("DROP TRIGGER IF EXISTS update_posts_updated_at ON posts")?;
        conn.execute("DROP FUNCTION IF EXISTS update_updated_at_column()")?;
        conn.execute("ALTER TABLE users DROP COLUMN IF EXISTS updated_at")?;
        conn.execute("ALTER TABLE posts DROP COLUMN IF EXISTS updated_at")?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Complete PostgreSQL Migration Example ===\n");
    
    // Get database URL from environment or use default
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://myuser:mypassword@localhost:5432/postgres".to_string());
    
    println!("Connecting to: {}", database_url);
    
    // Connect to PostgreSQL
    let mut client = match Client::connect(&database_url, NoTls) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            eprintln!("\nPlease ensure PostgreSQL is running and accessible.");
            eprintln!("Default connection: postgresql://myuser:mypassword@localhost:5432/postgres");
            eprintln!("Or set DATABASE_URL environment variable to your database.");
            return Ok(());
        }
    };
    
    // Configure migration runner
    let mut config = MigrationConfig::default();
    config.table.table_name = "schema_migrations".to_string();
    config.transaction_per_migration = true;
    config.auto_create_table = true;
    config.allow_out_of_order = false; // Gap detection now works correctly
    
    println!("Migration Configuration:");
    println!("  Table: {}", config.table.table_name);
    println!("  Transactions: {}", config.transaction_per_migration);
    println!("  Auto-create table: {}", config.auto_create_table);
    println!();
    
    let mut runner = MigrationRunner::with_config(config);
    
    // Add migrations
    runner.add_migration(Box::new(CreateUsersTable));
    runner.add_migration(Box::new(CreatePostsTable));
    runner.add_migration(Box::new(AddUpdatedAtTriggers));
    
    // Create migration connection
    let mut migration_conn = PostgresMigrationConnection::new(&mut client);
    
    // First run - apply all migrations
    println!("--- Applying Migrations ---");
    match runner.run(&mut migration_conn) {
        Ok(report) => {
            println!("✓ Migration run completed");
            println!("  Applied: {} migrations", report.successful_count());
            println!("  Skipped: {} migrations", report.skipped.len());
            println!("  Failed: {} migrations", report.failed_count());
            println!("  Total time: {}ms", report.total_time_ms);
            
            println!("\nMigration details:");
            for result in &report.successful {
                println!("  ✓ Version {} - {} ({}ms)", 
                    result.version, 
                    result.name, 
                    result.execution_time_ms
                );
            }
        }
        Err(e) => {
            eprintln!("✗ Error running migrations: {}", e);
            return Ok(());
        }
    }
    
    // Verify database schema
    println!("\n--- Database Schema ---");
    
    // List tables
    let tables = client.query(
        "SELECT table_name FROM information_schema.tables 
         WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
         ORDER BY table_name",
        &[]
    )?;
    
    println!("Tables:");
    for row in &tables {
        let table_name: String = row.get(0);
        println!("  - {}", table_name);
    }
    
    // List triggers
    let triggers = client.query(
        "SELECT trigger_name, event_object_table 
         FROM information_schema.triggers 
         WHERE trigger_schema = 'public'
         ORDER BY trigger_name",
        &[]
    )?;
    
    if !triggers.is_empty() {
        println!("\nTriggers:");
        for row in &triggers {
            let trigger_name: String = row.get(0);
            let table_name: String = row.get(1);
            println!("  - {} on {}", trigger_name, table_name);
        }
    }
    
    // Check migration status
    println!("\n--- Migration Status ---");
    let mut migration_conn = PostgresMigrationConnection::new(&mut client);
    
    match runner.status(&mut migration_conn) {
        Ok(statuses) => {
            println!("{:<10} {:<30} {:<15} {:<25}", "Version", "Name", "Status", "Applied At");
            println!("{:-<80}", "");
            
            for status in statuses {
                let status_text = if status.applied { "Applied" } else { "Pending" };
                let applied_at = status.applied_at
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "-".to_string());
                
                println!("{:<10} {:<30} {:<15} {:<25}", 
                    status.version, 
                    status.name, 
                    status_text, 
                    applied_at
                );
            }
        }
        Err(e) => eprintln!("Error getting status: {}", e),
    }
    
    // Test data insertion
    println!("\n--- Testing Database ---");
    
    // Insert test user
    client.execute(
        "INSERT INTO users (username, email) VALUES ($1, $2)",
        &[&"testuser", &"test@example.com"]
    )?;
    
    // Get user ID
    let user_id: i64 = client.query_one(
        "SELECT id FROM users WHERE username = $1",
        &[&"testuser"]
    )?.get(0);
    
    // Insert test post
    client.execute(
        "INSERT INTO posts (user_id, title, content) VALUES ($1, $2, $3)",
        &[&user_id, &"Test Post", &"This is a test post content"]
    )?;
    
    // Update user to test trigger
    client.execute(
        "UPDATE users SET email = $1 WHERE id = $2",
        &[&"newemail@example.com", &user_id]
    )?;
    
    // Verify trigger worked
    let row = client.query_one(
        "SELECT created_at, updated_at FROM users WHERE id = $1",
        &[&user_id]
    )?;
    
    let created_at: std::time::SystemTime = row.get(0);
    let updated_at: std::time::SystemTime = row.get(1);
    let created_at = chrono::DateTime::<chrono::Utc>::from(created_at);
    let updated_at = chrono::DateTime::<chrono::Utc>::from(updated_at);
    
    println!("  User created at: {}", created_at.format("%Y-%m-%d %H:%M:%S"));
    println!("  User updated at: {}", updated_at.format("%Y-%m-%d %H:%M:%S"));
    println!("  Trigger working: {}", updated_at > created_at);
    
    // Count records
    let user_count: i64 = client.query_one("SELECT COUNT(*) FROM users", &[])?.get(0);
    let post_count: i64 = client.query_one("SELECT COUNT(*) FROM posts", &[])?.get(0);
    
    println!("  Total users: {}", user_count);
    println!("  Total posts: {}", post_count);
    
    // Test rollback
    println!("\n--- Testing Rollback to Version 1 ---");
    let mut migration_conn = PostgresMigrationConnection::new(&mut client);
    match runner.rollback(&mut migration_conn, 1) {
        Ok(report) => {
            println!("✓ Rollback completed");
            println!("  Rolled back: {} migrations", report.successful_count());
            
            for result in &report.successful {
                println!("  ✓ Rolled back version {} - {}", result.version, result.name);
            }
        }
        Err(e) => eprintln!("Error during rollback: {}", e),
    }
    
    // Verify rollback
    println!("\n--- After Rollback ---");
    let tables_after = client.query(
        "SELECT table_name FROM information_schema.tables 
         WHERE table_schema = 'public' AND table_type = 'BASE TABLE' 
         AND table_name != 'schema_migrations'
         ORDER BY table_name",
        &[]
    )?;
    
    println!("Remaining tables:");
    for row in &tables_after {
        let table_name: String = row.get(0);
        println!("  - {}", table_name);
    }
    
    println!("\n✓ PostgreSQL migration example completed successfully!");
    println!("\nNote: Migrations have been applied to the database.");
    println!("Migration table: schema_migrations");
    
    Ok(())
}