//! SQLite Example for Parsql
//! 
//! This example demonstrates all features of parsql-sqlite:
//! - Basic CRUD operations
//! - Extension methods
//! - Transaction support
//! - Complex queries
//! - Prelude usage

use anyhow::Result;
use parsql::prelude::*;

// Define our models using derive macros
#[derive(Debug, Clone)]
struct User {
    id: i64,
    name: String,
    email: String,
    active: bool,
}

// Model for inserting users
#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
    active: bool,
}

// Model for querying a single user by ID
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("id = ?")]
struct GetUserById {
    id: i64,
    name: String,
    email: String,
    active: bool,
}

// Model for querying all active users
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("active = ?")]
struct GetActiveUsers {
    active: bool,
    id: i64,
    name: String,
    email: String,
}

// Model for updating user information
#[derive(Updateable, UpdateParams)]
#[table("users")]
#[update("name, email")]
#[where_clause("id = ?")]
struct UpdateUser {
    id: i64,
    name: String,
    email: String,
}

// Model for deleting a user
#[derive(Deletable, SqlParams)]
#[table("users")]
#[where_clause("id = ?")]
struct DeleteUser {
    id: i64,
}

// Complex query example with JOIN
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users u")]
#[select("u.id, u.name, u.email, COUNT(p.id) as post_count")]
#[join("LEFT JOIN posts p ON u.id = p.user_id")]
#[where_clause("u.active = ?")]
#[group_by("u.id, u.name, u.email")]
#[order_by("post_count DESC")]
struct GetUserWithPostCount {
    active: bool,
    id: i64,
    name: String,
    email: String,
    post_count: i64,
}

fn main() -> Result<()> {
    // Enable SQL tracing if needed
    // std::env::set_var("PARSQL_TRACE", "1");

    // Create an in-memory SQLite database
    let mut conn = Connection::open_in_memory()?;
    
    // Create tables
    create_tables(&conn)?;
    
    // Demonstrate function-based approach
    println!("=== Function-based Approach ===");
    demo_function_approach(&conn)?;
    
    // Demonstrate extension methods
    println!("\n=== Extension Methods Approach ===");
    demo_extension_methods(&conn)?;
    
    // Demonstrate transactions
    println!("\n=== Transaction Example ===");
    demo_transactions(&mut conn)?;
    
    // Demonstrate complex queries
    println!("\n=== Complex Query Example ===");
    demo_complex_queries(&conn)?;
    
    Ok(())
}

fn create_tables(conn: &Connection) -> Result<()> {
    // Create users table
    conn.execute(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            active BOOLEAN NOT NULL DEFAULT 1
        )",
        [],
    )?;
    
    // Create posts table for complex query example
    conn.execute(
        "CREATE TABLE posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            content TEXT,
            FOREIGN KEY (user_id) REFERENCES users (id)
        )",
        [],
    )?;
    
    println!("Tables created successfully");
    Ok(())
}

fn demo_function_approach(conn: &Connection) -> Result<()> {
    // Import function-based CRUD operations
    use parsql::sqlite::{insert, fetch, fetch_all};
    
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
    // Note: SQLite insert returns rows affected
    let rows1: usize = insert(conn, user1)?;
    let rows2: usize = insert(conn, user2)?;
    let rows3: usize = insert(conn, user3)?;
    
    println!("Inserted {} + {} + {} users", rows1, rows2, rows3);
    
    // Get the IDs of inserted records
    let id1 = conn.last_insert_rowid() - 2;
    let _id2 = conn.last_insert_rowid() - 1;
    let _id3 = conn.last_insert_rowid();
    
    // Fetch a single user
    let query = GetUserById {
        id: id1,
        name: String::new(),
        email: String::new(),
        active: false,
    };
    
    let user = fetch(conn, &query)?;
    println!("Fetched user: {:?}", user);
    
    // Fetch all active users
    let active_query = GetActiveUsers {
        active: true,
        id: 0,
        name: String::new(),
        email: String::new(),
    };
    
    let active_users = fetch_all(conn, &active_query)?;
    println!("Active users: {} found", active_users.len());
    for user in active_users {
        println!("  - {:?}", user);
    }
    
    Ok(())
}

fn demo_extension_methods(conn: &Connection) -> Result<()> {
    // Insert using extension method
    let new_user = InsertUser {
        name: "Diana Prince".to_string(),
        email: "diana@example.com".to_string(),
        active: true,
    };
    
    let rows: usize = conn.insert(new_user)?;
    println!("Inserted {} user(s)", rows);
    let id = conn.last_insert_rowid(); // Get the last inserted ID
    
    // Update using extension method
    let update = UpdateUser {
        id,
        name: "Diana Prince (Wonder Woman)".to_string(),
        email: "wonderwoman@example.com".to_string(),
    };
    
    let affected = conn.update(update)?;
    println!("Updated {} row(s)", affected);
    
    // Fetch using extension method
    let query = GetUserById {
        id,
        name: String::new(),
        email: String::new(),
        active: false,
    };
    
    let user = conn.fetch(&query)?;
    println!("Updated user: {:?}", user);
    
    // Delete using extension method
    let delete = DeleteUser { id };
    let deleted = conn.delete(delete)?;
    println!("Deleted {} row(s)", deleted);
    
    Ok(())
}

fn demo_transactions(conn: &mut Connection) -> Result<()> {
    // SQLite transactions
    let tx = conn.transaction()?;
    
    // Insert a user
    let user = InsertUser {
        name: "Eve Wilson".to_string(),
        email: "eve@example.com".to_string(),
        active: true,
    };
    
    let rows: usize = tx.insert(user)?;
    let id = tx.last_insert_rowid();
    println!("In transaction: Inserted {} user(s) with ID: {}", rows, id);
    
    // Update the user
    let update = UpdateUser {
        id,
        name: "Eve Wilson (Updated)".to_string(),
        email: "eve.updated@example.com".to_string(),
    };
    
    tx.update(update)?;
    println!("In transaction: Updated user");
    
    // Fetch to verify
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

fn demo_complex_queries(conn: &Connection) -> Result<()> {
    // Insert some posts for testing
    conn.execute(
        "INSERT INTO posts (user_id, title, content) VALUES 
         (1, 'First Post', 'Hello World'),
         (1, 'Second Post', 'Another post'),
         (2, 'Bob''s Post', 'Hi there')",
        [],
    )?;
    
    // Execute complex query with JOIN
    let query = GetUserWithPostCount {
        active: true,
        id: 0,
        name: String::new(),
        email: String::new(),
        post_count: 0,
    };
    let results = conn.fetch_all(&query)?;
    
    println!("Users with post counts:");
    for user in results {
        println!("  - {}: {} posts", user.name, user.post_count);
    }
    
    Ok(())
}