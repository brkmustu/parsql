//! Simple migration example to demonstrate the basic concepts.

use std::collections::HashMap;
use chrono::Utc;

/// Represents a migration
#[derive(Debug, Clone)]
struct Migration {
    version: i64,
    name: String,
    up_sql: String,
    down_sql: String,
}

impl Migration {
    fn new(version: i64, name: &str, up_sql: &str, down_sql: &str) -> Self {
        Self {
            version,
            name: name.to_string(),
            up_sql: up_sql.to_string(),
            down_sql: down_sql.to_string(),
        }
    }
}

/// Simple in-memory database for testing
#[derive(Default)]
struct TestDb {
    tables: HashMap<String, Vec<HashMap<String, String>>>,
    executed_queries: Vec<String>,
}

impl TestDb {
    fn execute(&mut self, sql: &str) -> Result<(), String> {
        println!("Executing: {}", sql);
        self.executed_queries.push(sql.to_string());
        
        // Simple parsing for demo
        if sql.starts_with("CREATE TABLE") {
            if let Some(table_name) = sql.split_whitespace().nth(2) {
                self.tables.insert(table_name.to_string(), Vec::new());
            }
        } else if sql.starts_with("DROP TABLE") {
            if let Some(table_name) = sql.split_whitespace().nth(4) {
                self.tables.remove(table_name);
            }
        }
        
        Ok(())
    }
    
    fn table_exists(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }
}

/// Migration runner
struct MigrationRunner {
    migrations: Vec<Migration>,
    applied: HashMap<i64, chrono::DateTime<Utc>>,
}

impl MigrationRunner {
    fn new() -> Self {
        Self {
            migrations: Vec::new(),
            applied: HashMap::new(),
        }
    }
    
    fn add_migration(&mut self, migration: Migration) {
        self.migrations.push(migration);
    }
    
    fn run(&mut self, db: &mut TestDb) -> Result<(), String> {
        // Sort by version
        self.migrations.sort_by_key(|m| m.version);
        
        for migration in &self.migrations {
            if self.applied.contains_key(&migration.version) {
                println!("Skipping migration {}: {} (already applied)", 
                    migration.version, migration.name);
                continue;
            }
            
            println!("Running migration {}: {}", migration.version, migration.name);
            
            // Execute up migration
            db.execute(&migration.up_sql)?;
            
            // Record as applied
            self.applied.insert(migration.version, Utc::now());
            
            println!("✓ Migration {} completed", migration.version);
        }
        
        Ok(())
    }
    
    fn rollback(&mut self, db: &mut TestDb, target_version: i64) -> Result<(), String> {
        // Sort by version (descending)
        self.migrations.sort_by_key(|m| std::cmp::Reverse(m.version));
        
        for migration in &self.migrations {
            if migration.version <= target_version {
                break;
            }
            
            if !self.applied.contains_key(&migration.version) {
                continue;
            }
            
            println!("Rolling back migration {}: {}", migration.version, migration.name);
            
            // Execute down migration
            db.execute(&migration.down_sql)?;
            
            // Remove from applied
            self.applied.remove(&migration.version);
            
            println!("✓ Rollback {} completed", migration.version);
        }
        
        Ok(())
    }
    
    fn status(&self) {
        println!("\nMigration Status:");
        println!("{:<10} {:<30} {:<10}", "Version", "Name", "Status");
        println!("{:-<50}", "");
        
        let mut all_migrations = self.migrations.clone();
        all_migrations.sort_by_key(|m| m.version);
        
        for migration in &all_migrations {
            let status = if self.applied.contains_key(&migration.version) {
                "Applied"
            } else {
                "Pending"
            };
            
            println!("{:<10} {:<30} {:<10}", migration.version, migration.name, status);
        }
    }
}

fn main() {
    println!("=== Simple Migration Example ===\n");
    
    let mut db = TestDb::default();
    let mut runner = MigrationRunner::new();
    
    // Define migrations
    runner.add_migration(Migration::new(
        1,
        "create_users_table",
        "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(255))",
        "DROP TABLE IF EXISTS users"
    ));
    
    runner.add_migration(Migration::new(
        2,
        "add_email_to_users",
        "ALTER TABLE users ADD COLUMN email VARCHAR(255)",
        "ALTER TABLE users DROP COLUMN email"
    ));
    
    runner.add_migration(Migration::new(
        3,
        "create_posts_table",
        "CREATE TABLE posts (id INT PRIMARY KEY, user_id INT, title VARCHAR(255))",
        "DROP TABLE IF EXISTS posts"
    ));
    
    // Show initial status
    runner.status();
    
    // Run migrations
    println!("\n--- Running Migrations ---");
    runner.run(&mut db).unwrap();
    
    // Show status after running
    runner.status();
    
    // Verify tables exist
    println!("\n--- Verification ---");
    println!("Users table exists: {}", db.table_exists("users"));
    println!("Posts table exists: {}", db.table_exists("posts"));
    
    // Rollback to version 1
    println!("\n--- Rolling Back to Version 1 ---");
    runner.rollback(&mut db, 1).unwrap();
    
    // Show final status
    runner.status();
    
    // Verify rollback
    println!("\n--- Final Verification ---");
    println!("Users table exists: {}", db.table_exists("users"));
    println!("Posts table exists: {}", db.table_exists("posts"));
    
    println!("\n--- Executed Queries ---");
    for (i, query) in db.executed_queries.iter().enumerate() {
        println!("{}. {}", i + 1, query);
    }
}