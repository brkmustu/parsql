//! PostgreSQL Example for Parsql
//! 
//! This example demonstrates all features of parsql-postgres:
//! - Basic CRUD operations
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
    id: i32,
    name: String,
    email: String,
    active: bool,
}

// Model for querying all active users
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("active = $")]
struct GetActiveUsers {
    active: bool,
    id: i32,
    name: String,
    email: String,
}

// Model for updating user information
#[derive(Updateable, UpdateParams)]
#[table("users")]
#[update("name, email")]
#[where_clause("id = $")]
struct UpdateUser {
    id: i32,
    name: String,
    email: String,
}

// Model for deleting a user
#[derive(Deletable, SqlParams)]
#[table("users")]
#[where_clause("id = $")]
struct DeleteUser {
    id: i32,
}

// Complex query example with JOIN and aggregation
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users u")]
#[select("u.id, u.name, u.email, COUNT(p.id) as post_count, MAX(p.created_at) as last_post_date")]
#[join("LEFT JOIN posts p ON u.id = p.user_id")]
#[where_clause("u.active = $ AND u.created_at > $")]
#[group_by("u.id, u.name, u.email")]
#[having("COUNT(p.id) > $")]
#[order_by("post_count DESC")]
#[limit(10)]
struct GetActiveUsersWithPosts {
    active: bool,
    created_after: chrono::NaiveDateTime,
    min_posts: i64,
    id: i32,
    name: String,
    email: String,
    post_count: i64,
    last_post_date: Option<chrono::NaiveDateTime>,
}

fn main() -> Result<()> {
    // Enable SQL tracing if needed
    // std::env::set_var("PARSQL_TRACE", "1");

    // Connect to PostgreSQL
    // Note: Update connection string with your PostgreSQL credentials
    let mut client = PostgresClient::connect(
        "host=localhost user=myuser password=mypassword dbname=parsql_test",
        PostgresNoTls,
    )?;
    
    // Create tables
    create_tables(&mut client)?;
    
    // Demonstrate function-based approach
    println!("=== Function-based Approach ===");
    demo_function_approach(&mut client)?;
    
    // Demonstrate extension methods
    println!("\n=== Extension Methods Approach ===");
    demo_extension_methods(&mut client)?;
    
    // Demonstrate transactions
    println!("\n=== Transaction Example ===");
    demo_transactions(&mut client)?;
    
    // Demonstrate complex queries
    println!("\n=== Complex Query Example ===");
    demo_complex_queries(&mut client)?;
    
    // Cleanup
    cleanup_tables(&mut client)?;
    
    Ok(())
}

fn create_tables(client: &mut PostgresClient) -> Result<()> {
    // Drop tables if they exist
    client.execute("DROP TABLE IF EXISTS posts CASCADE", &[])?;
    client.execute("DROP TABLE IF EXISTS users CASCADE", &[])?;
    
    // Create users table
    client.execute(
        "CREATE TABLE users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL UNIQUE,
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        &[],
    )?;
    
    // Create posts table for complex query example
    client.execute(
        "CREATE TABLE posts (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id),
            title VARCHAR(255) NOT NULL,
            content TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        &[],
    )?;
    
    println!("Tables created successfully");
    Ok(())
}

fn demo_function_approach(client: &mut PostgresClient) -> Result<()> {
    // Insert users using function approach
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
    
    let user3 = InsertUser {
        name: "Charlie Brown".to_string(),
        email: "charlie@example.com".to_string(),
        active: false,
    };
    
    // Using the function-based approach
    use parsql::postgres::{insert, fetch, fetch_all};
    let id1: i32 = insert(client, user1)?;
    let id2: i32 = insert(client, user2)?;
    let id3: i32 = insert(client, user3)?;
    
    println!("Inserted users with IDs: {}, {}, {}", id1, id2, id3);
    
    // Fetch a single user
    let query = GetUserById {
        id: id1,
        name: String::new(),
        email: String::new(),
        active: false,
    };
    
    let user = fetch(client, &query)?;
    println!("Fetched user: {:?}", user);
    
    // Fetch all active users
    let active_query = GetActiveUsers {
        active: true,
        id: 0,
        name: String::new(),
        email: String::new(),
    };
    
    let active_users = fetch_all(client, &active_query)?;
    println!("Active users: {} found", active_users.len());
    for user in active_users {
        println!("  - {:?}", user);
    }
    
    Ok(())
}

fn demo_extension_methods(client: &mut PostgresClient) -> Result<()> {
    // Insert using extension method
    let new_user = InsertUser {
        name: "Diana Prince".to_string(),
        email: "diana@example.com".to_string(),
        active: true,
    };
    
    let id: i32 = client.insert(new_user)?;
    println!("Inserted user with ID: {}", id);
    
    // Update using extension method
    let update = UpdateUser {
        id,
        name: "Diana Prince (Wonder Woman)".to_string(),
        email: "wonderwoman@example.com".to_string(),
    };
    
    let affected = client.update(update)?;
    println!("Updated {} row(s)", affected);
    
    // Fetch using extension method
    let query = GetUserById {
        id,
        name: String::new(),
        email: String::new(),
        active: false,
    };
    
    let user = client.fetch(&query)?;
    println!("Updated user: {:?}", user);
    
    // Delete using extension method
    let delete = DeleteUser { id };
    let deleted = client.delete(delete)?;
    println!("Deleted {} row(s)", deleted);
    
    Ok(())
}

fn demo_transactions(client: &mut PostgresClient) -> Result<()> {
    // Start a transaction
    let mut tx = client.transaction()?;
    
    // Insert a user within transaction
    let user = InsertUser {
        name: "Eve Wilson".to_string(),
        email: "eve@example.com".to_string(),
        active: true,
    };
    
    let id: i32 = tx.insert(user)?;
    println!("In transaction: Inserted user with ID: {}", id);
    
    // Update the user within transaction
    let update = UpdateUser {
        id,
        name: "Eve Wilson (Updated)".to_string(),
        email: "eve.updated@example.com".to_string(),
    };
    
    tx.update(update)?;
    println!("In transaction: Updated user");
    
    // Fetch to verify within transaction
    let query = GetUserById {
        id,
        name: String::new(),
        email: String::new(),
        active: false,
    };
    
    let user = tx.fetch(&query)?;
    println!("In transaction: User state: {:?}", user);
    
    // Commit the transaction
    tx.commit()?;
    println!("Transaction committed successfully");
    
    Ok(())
}

fn demo_complex_queries(client: &mut PostgresClient) -> Result<()> {
    // Insert some posts for testing
    client.execute(
        "INSERT INTO posts (user_id, title, content) VALUES 
         (1, 'First Post', 'Hello World'),
         (1, 'Second Post', 'Another post'),
         (1, 'Third Post', 'Yet another post'),
         (2, 'Bob''s Post', 'Hi there'),
         (2, 'Bob''s Second Post', 'More content')",
        &[],
    )?;
    
    // Update created_at for testing
    client.execute(
        "UPDATE users SET created_at = NOW() - INTERVAL '7 days' WHERE id <= 2",
        &[],
    )?;
    
    // Execute complex query with JOIN, GROUP BY, HAVING
    use chrono::{Utc, Duration};
    let one_month_ago = Utc::now().naive_utc() - Duration::days(30);
    
    let query = GetActiveUsersWithPosts {
        active: true,
        created_after: one_month_ago,
        min_posts: 1,
        id: 0,
        name: String::new(),
        email: String::new(),
        post_count: 0,
        last_post_date: None,
    };
    
    let results = client.fetch_all(&query)?;
    
    println!("Active users with posts (min 2 posts, created in last 30 days):");
    for user in results {
        println!(
            "  - {}: {} posts, last post: {:?}",
            user.name, user.post_count, user.last_post_date
        );
    }
    
    Ok(())
}

fn cleanup_tables(_client: &mut PostgresClient) -> Result<()> {
    // client.execute("DROP TABLE IF EXISTS posts CASCADE", &[])?;
    // client.execute("DROP TABLE IF EXISTS users CASCADE", &[])?;
    println!("\nTables preserved for inspection. Uncomment cleanup code to drop them.");
    Ok(())
}