//! Tokio PostgreSQL Example for Parsql
//! 
//! This example demonstrates all features of parsql-tokio-postgres:
//! - Async CRUD operations
//! - Extension methods
//! - Transaction support
//! - Complex queries
//! - Prelude usage

use anyhow::Result;
use parsql::prelude::*;

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

// Batch insert example
#[derive(Insertable, SqlParams)]
#[table("posts")]
#[returning("id")]
struct InsertPost {
    user_id: i64,
    title: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Enable SQL tracing if needed
    // std::env::set_var("PARSQL_TRACE", "1");

    // Connect to PostgreSQL
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=myuser password=mypassword dbname=parsql_test",
        TokioPostgresNoTls,
    )
    .await?;
    
    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    // Create tables
    create_tables(&client).await?;
    
    // Demonstrate async operations
    println!("=== Async CRUD Operations ===");
    demo_async_crud(&client).await?;
    
    // Demonstrate extension methods
    println!("\n=== Extension Methods ===");
    demo_extension_methods(&client).await?;
    
    // Demonstrate transactions
    println!("\n=== Transaction Example ===");
    demo_transactions(&client).await?;
    
    // Demonstrate batch operations
    println!("\n=== Batch Operations ===");
    demo_batch_operations(&client).await?;
    
    // Cleanup
    cleanup_tables(&client).await?;
    
    Ok(())
}

async fn create_tables(client: &TokioPostgresClient) -> Result<()> {
    // Drop tables if they exist
    client.execute("DROP TABLE IF EXISTS posts CASCADE", &[]).await?;
    client.execute("DROP TABLE IF EXISTS users CASCADE", &[]).await?;
    
    // Create users table
    client
        .execute(
            "CREATE TABLE users (
                id BIGSERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL UNIQUE,
                active BOOLEAN NOT NULL DEFAULT true,
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
    
    println!("Tables created successfully");
    Ok(())
}

async fn demo_async_crud(client: &TokioPostgresClient) -> Result<()> {
    // Insert users asynchronously
    let user1 = InsertUser {
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        active: true,
    };
    
    let user2 = InsertUser {
        name: "Bob Smith".to_string(),
        email: "bob@example.com".to_string(),
        active: true,
    };
    
    // Async insert operations
    let id1: i64 = insert(client, user1).await?;
    let id2: i64 = insert(client, user2).await?;
    
    println!("Inserted users with IDs: {}, {}", id1, id2);
    
    // Fetch a single user
    let query = GetUserById {
        id: id1,
        name: String::new(),
        email: String::new(),
        active: false,
    };
    
    let user = fetch(client, query).await?;
    println!("Fetched user: {:?}", user);
    
    // Fetch all users
    let all_query = GetAllUsers {
        id: 0,
        name: String::new(),
        email: String::new(),
        active: false,
    };
    
    let all_users = fetch_all(client, all_query).await?;
    println!("All users: {} found", all_users.len());
    for user in all_users {
        println!("  - {:?}", user);
    }
    
    Ok(())
}

async fn demo_extension_methods(client: &TokioPostgresClient) -> Result<()> {
    // Insert using extension method
    let new_user = InsertUser {
        name: "Diana Prince".to_string(),
        email: "diana@example.com".to_string(),
        active: true,
    };
    
    let id: i64 = client.insert(new_user).await?;
    println!("Inserted user with ID: {}", id);
    
    // Update using extension method
    let update = UpdateUser {
        id,
        name: "Diana Prince (Wonder Woman)".to_string(),
        email: "wonderwoman@example.com".to_string(),
    };
    
    let affected = client.update(update).await?;
    println!("Updated {} row(s)", affected);
    
    // Fetch using extension method
    let query = GetUserById {
        id,
        name: String::new(),
        email: String::new(),
        active: false,
    };
    
    let user = client.fetch(query).await?;
    println!("Updated user: {:?}", user);
    
    // Delete using extension method
    let delete = DeleteUser { id };
    let deleted = client.delete(delete).await?;
    println!("Deleted {} row(s)", deleted);
    
    Ok(())
}

async fn demo_transactions(client: &TokioPostgresClient) -> Result<()> {
    // Note: In production, you'd typically have a mutable client reference
    // For demo purposes, we'll skip transaction rollback demonstration
    println!("Note: Transaction demo simplified for tokio-postgres example.");
    
    // In tokio-postgres, transactions require a mutable client
    // For a proper example, see the sync postgres example
    println!("For full transaction examples, see postgres-example");
    
    // We can still demonstrate the CRUD operations work correctly
    let user = InsertUser {
        name: "Eve Wilson".to_string(),
        email: "eve@example.com".to_string(),
        active: true,
    };
    
    let id: i64 = client.insert(user).await?;
    println!("Inserted user with ID: {}", id);
    
    // Update the user
    let update = UpdateUser {
        id,
        name: "Eve Wilson (Updated)".to_string(),
        email: "eve.updated@example.com".to_string(),
    };
    
    client.update(update).await?;
    println!("Updated user");
    
    // Fetch to verify
    let query = GetUserById {
        id,
        name: String::new(),
        email: String::new(),
        active: false,
    };
    
    let user = client.fetch(query).await?;
    println!("User state: {:?}", user);
    
    // Clean up
    let delete = DeleteUser { id };
    client.delete(delete).await?;
    println!("Deleted user");
    
    Ok(())
}

async fn demo_batch_operations(client: &TokioPostgresClient) -> Result<()> {
    // Get all users for batch insert
    let all_query = GetAllUsers {
        id: 0,
        name: String::new(),
        email: String::new(),
        active: false,
    };
    
    let users = client.fetch_all(all_query).await?;
    
    // Batch insert posts
    println!("Inserting posts for {} users", users.len());
    
    for user in users {
        let posts = vec![
            InsertPost {
                user_id: user.id,
                title: format!("{}'s First Post", user.name),
                content: "Hello from async Rust!".to_string(),
            },
            InsertPost {
                user_id: user.id,
                title: format!("{}'s Second Post", user.name),
                content: "Tokio PostgreSQL is awesome!".to_string(),
            },
        ];
        
        for post in posts {
            let post_id: i64 = client.insert(post).await?;
            println!("  - Created post {} for user {}", post_id, user.name);
        }
    }
    
    Ok(())
}

async fn cleanup_tables(_client: &TokioPostgresClient) -> Result<()> {
    // client.execute("DROP TABLE IF EXISTS posts CASCADE", &[]).await?;
    // client.execute("DROP TABLE IF EXISTS users CASCADE", &[]).await?;
    println!("\nTables preserved for inspection. Uncomment cleanup code to drop them.");
    Ok(())
}