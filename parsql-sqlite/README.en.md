# parsql-sqlite

SQLite integration crate for parsql. This package provides synchronous APIs that enable parsql to work with SQLite databases.

## Features

- Synchronous SQLite operations
- Automatic SQL query generation
- Secure parameter management
- Generic CRUD operations (get, insert, update)
- Conversion of database rows to structs

## Installation

Add to your Cargo.toml file as follows:

```toml
[dependencies]
parsql = { version = "0.3.0", features = ["sqlite"] }
```

or if you want to use this package directly:

```toml
[dependencies]
parsql-sqlite = "0.3.0"
parsql-macros = "0.3.0"
```

## Basic Usage

This package uses **synchronous operations** when working with SQLite databases. This means it doesn't require async/await usage.

### Establishing a Connection

```rust
use rusqlite::Connection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create SQLite connection
    let conn = Connection::open("database.db")?;
    
    // Create example table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            state INTEGER NOT NULL
        )",
        [],
    )?;
    
    // ...
    
    Ok(())
}
```

## CRUD Operations

### Reading Data (Get)

```rust
use parsql::{
    core::Queryable,
    macros::{FromRow, Queryable, SqlParams},
    sqlite::{FromRow, SqlParams, get},
};
use rusqlite::{types::ToSql, Row};

#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("id = $")]
pub struct GetUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}

impl GetUser {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            name: Default::default(),
            email: Default::default(),
            state: Default::default(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("database.db")?;
    
    // Usage
    let get_user = GetUser::new(1);
    let get_result = get(&conn, get_user)?;
    
    println!("User: {:?}", get_result);
    Ok(())
}
```

### Adding Data (Insert)

```rust
use parsql::{
    core::Insertable,
    macros::{Insertable, SqlParams},
    sqlite::{SqlParams, insert},
};
use rusqlite::types::ToSql;

#[derive(Insertable)]
#[table("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("database.db")?;
    
    let insert_user = InsertUser {
        name: "Ali".to_string(),
        email: "ali@parsql.com".to_string(),
        state: 1,
    };
    
    let insert_result = insert(&conn, insert_user)?;
    println!("Inserted record ID: {}", insert_result);
    
    Ok(())
}
```

### Updating Data (Update)

```rust
use parsql::{
    core::Updateable,
    macros::{UpdateParams, Updateable},
    sqlite::{UpdateParams, update},
};
use rusqlite::types::ToSql;

#[derive(Updateable, UpdateParams)]
#[table("users")]
#[update("name, email")]
#[where_clause("id = $")]
pub struct UpdateUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("database.db")?;
    
    let update_user = UpdateUser {
        id: 1,
        name: String::from("Ali Updated"),
        email: String::from("ali.updated@gmail.com"),
        state: 2,
    };
    
    let result = update(&conn, update_user)?;
    println!("Number of records updated: {}", result);
    
    Ok(())
}
```

### Deleting Data (Delete)

```rust
use parsql::{
    core::Deletable,
    macros::{Deletable, SqlParams},
    sqlite::{SqlParams, delete},
};
use rusqlite::types::ToSql;

#[derive(Deletable, SqlParams)]
#[table("users")]
#[where_clause("id = $")]
pub struct DeleteUser {
    pub id: i64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("database.db")?;
    
    let delete_user = DeleteUser { id: 1 };
    let result = delete(&conn, delete_user)?;
    
    println!("Number of records deleted: {}", result);
    Ok(())
}
```

## Advanced Features

### Using Joins

```rust
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[select("users.id, users.name, posts.title as post_title")]
#[join("LEFT JOIN posts ON users.id = posts.user_id")]
#[where_clause("users.id = $")]
pub struct UserWithPosts {
    pub id: i64,
    pub name: String,
    pub post_title: Option<String>,
}
```

### Grouping and Ordering

```rust
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[select("state, COUNT(*) as user_count")]
#[group_by("state")]
#[order_by("user_count DESC")]
#[having("COUNT(*) > 5")]
pub struct UserStats {
    pub state: i16,
    pub user_count: i64,
}
```

### Custom Select Statements

```rust
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[select("id, name, email, CASE WHEN state = 1 THEN 'Active' ELSE 'Inactive' END as status")]
#[where_clause("id = $")]
pub struct UserWithStatus {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub status: String,
}
```

## SQL Query Tracing

To see the SQL queries being generated, you can set the `PARSQL_TRACE` environment variable:

```sh
PARSQL_TRACE=1 cargo run
```

This will print all queries generated for SQLite to the console.

## Performance Tips

1. **Indexing**: To improve the performance of your SQLite queries, create indexes on columns you frequently query.

   ```sql
   CREATE INDEX idx_users_email ON users(email);
   ```

2. **Batch Operations**: When performing multiple insert, update, or delete operations, do them within a transaction:

   ```rust
   conn.execute("BEGIN TRANSACTION", [])?;
   // Perform your operations here
   conn.execute("COMMIT", [])?;
   ```

3. **Prepared Statements**: Parsql already uses prepared statements under the hood, which helps protect against SQL injection attacks.

## Error Handling

Use Rust's `Result` mechanism to catch and handle errors that may occur during SQLite operations:

```rust
match get(&conn, get_user) {
    Ok(user) => println!("User found: {:?}", user),
    Err(e) => eprintln!("Error occurred: {}", e),
}
```

## Complete Example Project

For a complete example project, see the [examples/sqlite](../examples/sqlite) directory in the main parsql repository. 