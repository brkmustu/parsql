//! Deadpool PostgreSQL Example for Parsql
//! 
//! This example demonstrates all features of parsql-deadpool-postgres:
//! - Connection pool management
//! - Async CRUD operations through pool
//! - Transaction support with pooled connections
//! - Concurrent operations leveraging the pool
//! - Complex queries
//! - Prelude usage

use anyhow::Result;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use parsql::prelude::*;
use tokio_postgres::{Config, NoTls};
use std::time::Duration;

// Model for inserting users
#[derive(Insertable, SqlParams)]
#[table("users")]
#[returning("id")]
struct InsertUser {
    name: String,
    email: String,
    active: bool,
}

// Model for querying a single user by ID
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("id = $")]
struct GetUserById {
    id: i64,
    name: String,
    email: String,
    active: bool,
}

// Model for querying all users
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("")]  // Empty where clause for all records
struct GetAllUsers {
    id: i64,
    name: String,
    email: String,
    active: bool,
}

// Model for updating user information
#[derive(Updateable, UpdateParams)]
#[table("users")]
#[update("name, email")]
#[where_clause("id = $")]
struct UpdateUser {
    id: i64,
    name: String,
    email: String,
}

// Model for deleting a user
#[derive(Deletable, SqlParams)]
#[table("users")]
#[where_clause("id = $")]
struct DeleteUser {
    id: i64,
}

// Model for querying active users with pagination
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("active = $")]
#[order_by("id ASC")]
#[limit(5)]
#[offset(0)]
struct GetActiveUsersPaginated {
    active: bool,
    id: i64,
    name: String,
    email: String,
}

// Complex query with statistics
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users u")]
#[select("u.department, COUNT(*) as user_count, AVG(p.post_count) as avg_posts_per_user")]
#[join("LEFT JOIN (SELECT user_id, COUNT(*) as post_count FROM posts GROUP BY user_id) p ON u.id = p.user_id")]
#[where_clause("u.active = $")]
#[group_by("u.department")]
#[having("COUNT(*) > $")]
#[order_by("user_count DESC")]
struct GetDepartmentStats {
    active: bool,
    min_users: i64,
    department: String,
    user_count: i64,
    avg_posts_per_user: Option<f64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Enable SQL tracing if needed
    // std::env::set_var("PARSQL_TRACE", "1");

    // Create connection pool
    let pool = create_pool().await?;
    
    // Initialize database
    initialize_database(&pool).await?;
    
    // Demonstrate pool-based operations
    println!("=== Pool-based CRUD Operations ===");
    demo_pool_crud(&pool).await?;
    
    // Demonstrate concurrent operations
    println!("\n=== Concurrent Operations ===");
    demo_concurrent_operations(&pool).await?;
    
    // Demonstrate transaction management
    println!("\n=== Transaction Management ===");
    demo_transactions(&pool).await?;
    
    // Demonstrate pool statistics
    println!("\n=== Pool Statistics ===");
    demo_pool_statistics(&pool).await?;
    
    // Cleanup
    cleanup_database(&pool).await?;
    
    Ok(())
}

async fn create_pool() -> Result<Pool> {
    // Configure PostgreSQL connection
    let mut cfg = Config::new();
    cfg.host("localhost")
        .user("myuser")
        .password("mypassword")
        .dbname("parsql_test")
        .connect_timeout(Duration::from_secs(5));
    
    // Configure pool manager
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let mgr = Manager::from_config(cfg, NoTls, mgr_config);
    
    // Create pool with specific settings
    let pool = Pool::builder(mgr)
        .max_size(16)
        .build()?;
    
    println!("Connection pool created with max size: 16");
    Ok(pool)
}

async fn initialize_database(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;
    
    // Drop tables if they exist
    client.execute("DROP TABLE IF EXISTS posts CASCADE", &[]).await?;
    client.execute("DROP TABLE IF EXISTS users CASCADE", &[]).await?;
    
    // Create users table with department for complex queries
    client
        .execute(
            "CREATE TABLE users (
                id BIGSERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL UNIQUE,
                active BOOLEAN NOT NULL DEFAULT true,
                department VARCHAR(100) NOT NULL DEFAULT 'General',
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            &[],
        )
        .await?;
    
    // Create posts table
    client
        .execute(
            "CREATE TABLE posts (
                id BIGSERIAL PRIMARY KEY,
                user_id BIGINT NOT NULL REFERENCES users(id),
                title VARCHAR(255) NOT NULL,
                content TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            &[],
        )
        .await?;
    
    println!("Database initialized successfully");
    Ok(())
}

async fn demo_pool_crud(pool: &Pool) -> Result<()> {
    // Insert users with different departments
    let users = vec![
        ("Alice Johnson", "alice@example.com", "Engineering"),
        ("Bob Smith", "bob@example.com", "Engineering"),
        ("Charlie Brown", "charlie@example.com", "Marketing"),
        ("Diana Prince", "diana@example.com", "Marketing"),
        ("Eve Wilson", "eve@example.com", "Sales"),
    ];
    
    let mut user_ids = Vec::new();
    for (name, email, dept) in users {
        let user = InsertUser {
            name: name.to_string(),
            email: email.to_string(),
            active: true,
        };
        
        let id: i64 = parsql::deadpool_postgres::insert(pool, user).await?;
        
        // Update department
        let client = pool.get().await?;
        client.execute(
            "UPDATE users SET department = $1 WHERE id = $2",
            &[&dept, &id],
        ).await?;
        
        user_ids.push(id);
        println!("Inserted user {} with ID: {}", name, id);
    }
    
    // Fetch a single user
    let query = GetUserById {
        id: user_ids[0],
        name: String::new(),
        email: String::new(),
        active: false,
    };
    
    let user = parsql::deadpool_postgres::fetch(pool, &query).await?;
    println!("\nFetched user: {:?}", user);
    
    // Fetch all users
    let all_query = GetAllUsers {
        id: 0,
        name: String::new(),
        email: String::new(),
        active: false,
    };
    
    let all_users = parsql::deadpool_postgres::fetch_all(pool, &all_query).await?;
    println!("\nAll users: {} found", all_users.len());
    
    // Update a user
    let update_user = UpdateUser {
        id: user_ids[0],
        name: "Alice Johnson (Updated)".to_string(),
        email: "alice.updated@example.com".to_string(),
    };
    
    let affected = parsql::deadpool_postgres::update(pool, update_user).await?;
    println!("\nUpdated {} row(s)", if affected { 1 } else { 0 });
    
    Ok(())
}

async fn demo_concurrent_operations(pool: &Pool) -> Result<()> {
    use tokio::task::JoinSet;
    
    let mut tasks = JoinSet::new();
    
    // Spawn multiple concurrent operations
    for i in 0..5 {
        let pool = pool.clone();
        
        tasks.spawn(async move {
            // Each task inserts a user and some posts
            let user = InsertUser {
                name: format!("Concurrent User {}", i),
                email: format!("concurrent{}@example.com", i),
                active: true,
            };
            
            let user_id: i64 = parsql::deadpool_postgres::insert(&pool, user).await?;
            
            // Insert posts for this user
            let client = pool.get().await?;
            for j in 0..3 {
                client.execute(
                    "INSERT INTO posts (user_id, title, content) VALUES ($1, $2, $3)",
                    &[
                        &user_id,
                        &format!("Post {} by User {}", j, i),
                        &"Concurrent content",
                    ],
                ).await?;
            }
            
            println!("Task {} completed: created user {} with 3 posts", i, user_id);
            Ok::<_, anyhow::Error>(user_id)
        });
    }
    
    // Wait for all tasks to complete
    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        results.push(result??);
    }
    
    println!("\nAll concurrent operations completed. Created {} users", results.len());
    
    // Check pool statistics after concurrent usage
    let status = pool.status();
    println!("Pool status - Available: {}, Size: {}", status.available, status.size);
    
    Ok(())
}

async fn demo_transactions(pool: &Pool) -> Result<()> {
    // Get connection from pool for transaction
    let mut client = pool.get().await?;
    
    // Start transaction
    let tx = client.transaction().await?;
    
    println!("Starting transaction...");
    
    // Insert user in transaction
    let user = InsertUser {
        name: "Transaction Test User".to_string(),
        email: "tx_test@example.com".to_string(),
        active: true,
    };
    
    // Use transaction extensions from parsql
    use parsql::deadpool_postgres::transaction_extensions::TransactionExtensions;
    
    let id: i64 = tx.insert(user).await?;
    println!("In transaction: Inserted user with ID: {}", id);
    
    // Update in transaction
    let update_tx = UpdateUser {
        id,
        name: "Transaction Test User (Updated)".to_string(),
        email: "tx_test_updated@example.com".to_string(),
    };
    
    tx.update(update_tx).await?;
    println!("In transaction: Updated user");
    
    // Insert posts in transaction
    for i in 0..3 {
        tx.execute(
            "INSERT INTO posts (user_id, title, content) VALUES ($1, $2, $3)",
            &[
                &id,
                &format!("Transaction Post {}", i),
                &"Content from transaction",
            ],
        ).await?;
    }
    println!("In transaction: Created 3 posts");
    
    // Verify within transaction
    let query = GetUserById {
        id,
        name: String::new(),
        email: String::new(),
        active: false,
    };
    
    let user = tx.fetch(query).await?;
    println!("In transaction: User state: {:?}", user);
    
    // Commit transaction
    tx.commit().await?;
    println!("Transaction committed successfully");
    
    // Demonstrate rollback with new connection
    println!("\nDemonstrating rollback...");
    let mut client = pool.get().await?;
    let tx = client.transaction().await?;
    
    let user = InsertUser {
        name: "Rollback Test".to_string(),
        email: "rollback@example.com".to_string(),
        active: true,
    };
    
    let rollback_id: i64 = tx.insert(user).await?;
    println!("In transaction: Inserted user {} (will be rolled back)", rollback_id);
    
    tx.rollback().await?;
    println!("Transaction rolled back");
    
    // Verify rollback with new connection
    let query = GetUserById {
        id: rollback_id,
        name: String::new(),
        email: String::new(),
        active: false,
    };
    
    match parsql::deadpool_postgres::fetch(pool, &query).await {
        Ok(_) => println!("ERROR: User found after rollback!"),
        Err(_) => println!("Confirmed: User not found after rollback"),
    }
    
    Ok(())
}

async fn demo_pool_statistics(pool: &Pool) -> Result<()> {
    let status = pool.status();
    
    println!("Pool Statistics:");
    println!("  - Max Size: {}", status.max_size);
    println!("  - Current Size: {}", status.size);
    println!("  - Available Connections: {}", status.available);
    
    // Test complex query with statistics
    let stats_query = GetDepartmentStats {
        active: true,
        min_users: 1,
        department: String::new(),
        user_count: 0,
        avg_posts_per_user: None,
    };
    
    let results = parsql::deadpool_postgres::fetch_all(pool, &stats_query).await?;
    
    println!("\nDepartment Statistics:");
    for stat in results {
        println!(
            "  - {}: {} users, avg {:.2} posts per user",
            stat.department,
            stat.user_count,
            stat.avg_posts_per_user.unwrap_or(0.0)
        );
    }
    
    Ok(())
}

async fn cleanup_database(pool: &Pool) -> Result<()> {
    let _client = pool.get().await?;
    
    // _client.execute("DROP TABLE IF EXISTS posts CASCADE", &[]).await?;
    // _client.execute("DROP TABLE IF EXISTS users CASCADE", &[]).await?;
    
    println!("\nTables preserved for inspection. Uncomment cleanup code to drop them.");
    println!("Pool will be dropped when application exits.");
    
    Ok(())
}