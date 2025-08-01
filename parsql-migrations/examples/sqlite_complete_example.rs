//! Complete SQLite migration example demonstrating all features

use parsql_migrations::prelude::*;
use parsql_migrations::sqlite_simple::SqliteMigrationConnection;
use rusqlite::{Connection, Result as SqliteResult};

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
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )"
        )
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute("DROP TABLE IF EXISTS users")
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
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                content TEXT,
                published BOOLEAN DEFAULT FALSE,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES users(id)
            )"
        )
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute("DROP TABLE IF EXISTS posts")
    }
}

// Migration 3: Add indexes
struct AddIndexes;

impl Migration for AddIndexes {
    fn version(&self) -> i64 {
        3
    }
    
    fn name(&self) -> &str {
        "add_indexes"
    }
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute("CREATE INDEX idx_posts_user_id ON posts(user_id)")?;
        conn.execute("CREATE INDEX idx_posts_published ON posts(published)")?;
        conn.execute("CREATE INDEX idx_posts_created_at ON posts(created_at DESC)")?;
        Ok(())
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute("DROP INDEX IF EXISTS idx_posts_user_id")?;
        conn.execute("DROP INDEX IF EXISTS idx_posts_published")?;
        conn.execute("DROP INDEX IF EXISTS idx_posts_created_at")?;
        Ok(())
    }
}

// Migration 4: Add user profiles
struct AddUserProfiles;

impl Migration for AddUserProfiles {
    fn version(&self) -> i64 {
        4
    }
    
    fn name(&self) -> &str {
        "add_user_profiles"
    }
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute(
            "CREATE TABLE user_profiles (
                user_id INTEGER PRIMARY KEY,
                bio TEXT,
                avatar_url TEXT,
                website TEXT,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )"
        )
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute("DROP TABLE IF EXISTS user_profiles")
    }
}

fn main() -> SqliteResult<()> {
    println!("=== Complete SQLite Migration Example ===\n");
    
    // Create a new SQLite database
    let mut conn = Connection::open("complete_example.db")?;
    
    // Configure migration runner
    let mut config = MigrationConfig::default();
    config.table.table_name = "schema_migrations".to_string();
    config.transaction_per_migration = true;
    config.auto_create_table = true;
    config.allow_out_of_order = false; // Gap detection now works correctly
    config.verify_checksums = false; // Disable checksums for this example
    
    println!("Migration Configuration:");
    println!("  Table: {}", config.table.table_name);
    println!("  Transactions: {}", config.transaction_per_migration);
    println!("  Auto-create table: {}", config.auto_create_table);
    println!("  Allow out-of-order: {}", config.allow_out_of_order);
    println!();
    
    let mut runner = MigrationRunner::with_config(config);
    
    // Add all migrations
    runner.add_migration(Box::new(CreateUsersTable));
    runner.add_migration(Box::new(CreatePostsTable));
    runner.add_migration(Box::new(AddIndexes));
    runner.add_migration(Box::new(AddUserProfiles));
    
    // Create migration connection wrapper
    let mut migration_conn = SqliteMigrationConnection::new(&mut conn);
    
    // First run - apply all migrations
    println!("--- First Run: Applying All Migrations ---");
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
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")?
        .query_map([], |row| row.get(0))?
        .collect::<SqliteResult<Vec<_>>>()?;
    
    println!("Tables:");
    for table in &tables {
        println!("  - {}", table);
    }
    
    // List indexes
    let indexes: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%' ORDER BY name")?
        .query_map([], |row| row.get(0))?
        .collect::<SqliteResult<Vec<_>>>()?;
    
    if !indexes.is_empty() {
        println!("\nIndexes:");
        for index in &indexes {
            println!("  - {}", index);
        }
    }
    
    // Check migration status
    println!("\n--- Migration Status ---");
    let mut migration_conn = SqliteMigrationConnection::new(&mut conn);
    
    match runner.status(&mut migration_conn) {
        Ok(statuses) => {
            println!("{:<10} {:<25} {:<15} {:<25}", "Version", "Name", "Status", "Applied At");
            println!("{:-<75}", "");
            
            for status in statuses {
                let status_text = if status.applied { "Applied" } else { "Pending" };
                let applied_at = status.applied_at
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "-".to_string());
                
                println!("{:<10} {:<25} {:<15} {:<25}", 
                    status.version, 
                    status.name, 
                    status_text, 
                    applied_at
                );
            }
        }
        Err(e) => eprintln!("Error getting status: {}", e),
    }
    
    // Second run - should skip all
    println!("\n--- Second Run: Should Skip All ---");
    let mut migration_conn = SqliteMigrationConnection::new(&mut conn);
    match runner.run(&mut migration_conn) {
        Ok(report) => {
            println!("✓ Migration run completed");
            println!("  Applied: {} migrations", report.successful_count());
            println!("  Skipped: {} migrations", report.skipped.len());
            
            if !report.skipped.is_empty() {
                print!("  Skipped versions: ");
                for (i, version) in report.skipped.iter().enumerate() {
                    if i > 0 { print!(", "); }
                    print!("{}", version);
                }
                println!();
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    
    // Test rollback
    println!("\n--- Rollback Test: Rolling Back to Version 2 ---");
    let mut migration_conn = SqliteMigrationConnection::new(&mut conn);
    match runner.rollback(&mut migration_conn, 2) {
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
    let tables_after: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != 'schema_migrations' ORDER BY name")?
        .query_map([], |row| row.get(0))?
        .collect::<SqliteResult<Vec<_>>>()?;
    
    println!("Remaining tables:");
    for table in &tables_after {
        println!("  - {}", table);
    }
    
    // Final status
    println!("\n--- Final Status ---");
    let mut migration_conn = SqliteMigrationConnection::new(&mut conn);
    match runner.status(&mut migration_conn) {
        Ok(statuses) => {
            for status in statuses {
                let status_text = if status.applied { "Applied" } else { "Rolled back" };
                println!("  Version {}: {} - {}", status.version, status.name, status_text);
            }
        }
        Err(e) => eprintln!("Error getting status: {}", e),
    }
    
    // Insert some test data
    println!("\n--- Testing Database ---");
    conn.execute(
        "INSERT INTO users (username, email) VALUES (?, ?)",
        ["testuser", "test@example.com"]
    )?;
    
    conn.execute(
        "INSERT INTO posts (user_id, title, content) VALUES (?, ?, ?)",
        [&1.to_string(), "Test Post", "This is a test post"]
    )?;
    
    let user_count: i32 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    let post_count: i32 = conn.query_row("SELECT COUNT(*) FROM posts", [], |row| row.get(0))?;
    
    println!("  Users: {}", user_count);
    println!("  Posts: {}", post_count);
    
    println!("\n✓ Complete SQLite migration example finished successfully!");
    
    // Clean up
    std::fs::remove_file("complete_example.db").ok();
    
    Ok(())
}