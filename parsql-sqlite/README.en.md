# parsql-sqlite

SQLite integration crate for parsql. This package provides synchronous APIs that enable parsql to work with SQLite databases.

## Features

- Synchronous SQLite operations
- Automatic SQL query generation
- Secure parameter management
- Generic CRUD operations (fetch, insert, update)
- Conversion of database rows to structs
- Automatic protection against SQL Injection attacks
- Transaction support

## Security Features

### SQL Injection Protection

parsql-sqlite is designed to be secure against SQL Injection attacks:

- All user inputs are automatically parameterized
- SQLite's "?" parameterization structure is used automatically
- Macros process SQL parameters securely, providing protection against injection attacks
- Parameters are automatically managed to ensure correct order and type
- User inputs in `#[where_clause]` and other SQL components are always parameterized

```rust
// SQL injection protection example
#[derive(Queryable, FromRow, SqlParams)]
#[table("users")]
#[where_clause("username = ? AND status = ?")]
struct UserQuery {
    username: String,
    status: i32,
}

// User input (potentially harmful) is used safely
let query = UserQuery {
    username: user_input, // This value is not directly inserted into SQL, it's parameterized
    status: 1,
};

// Generated query: "SELECT * FROM users WHERE username = ? AND status = ?"
// Parameters are safely sent as: [user_input, 1]
let user = fetch(&conn, &query)?;
```

## Installation

Add to your Cargo.toml file as follows:

```toml
[dependencies]
parsql = { version = "0.3.2", features = ["sqlite"] }
```

or if you want to use this package directly:

```toml
[dependencies]
parsql-sqlite = "0.3.2"
parsql-macros = "0.3.2"
```

## Usage

parsql-sqlite offers two different approaches for working with SQLite databases:

1. Function-based approach (`fetch`, `insert`, `update`, etc.)
2. Extension method approach (`conn.fetch()`, `conn.insert()`, etc.) - `CrudOps` trait

### Establishing a Connection

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    // Create SQLite database connection
    let conn = Connection::open("test.db")?;

    // Create example table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL
        )",
        [],
    )?;
    
    // ...
    
    Ok(())
}
```

### Function-Based Approach

```rust
use rusqlite::{Connection, Result};
use parsql::sqlite::{fetch, insert};
use parsql::macros::{Insertable, SqlParams, Queryable, FromRow};

#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
}

#[derive(Queryable, FromRow, SqlParams)]
#[table("users")]
#[where_clause("id = ?")]
struct GetUser {
    id: i64,
    name: String,
    email: String,
}

fn main() -> Result<()> {
    let conn = Connection::open("test.db")?;
    
    // Add a user with function approach
    let insert_user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    let rows_affected = insert(&conn, insert_user)?;
    
    // Get a user with function approach
    let get_user = GetUser {
        id: 1,
        name: String::new(),
        email: String::new(),
    };
    let user = fetch(&conn, &get_user)?;
    
    println!("User: {:?}", user);
    Ok(())
}
```

### Extension Method Approach (CrudOps Trait)

```rust
use rusqlite::{Connection, Result};
use parsql::sqlite::CrudOps;  // Import the CrudOps trait
use parsql::macros::{Insertable, SqlParams, Queryable, FromRow};

#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
}

#[derive(Queryable, FromRow, SqlParams)]
#[table("users")]
#[where_clause("id = ?")]
struct GetUser {
    id: i64,
    name: String,
    email: String,
}

fn main() -> Result<()> {
    let conn = Connection::open("test.db")?;
    
    // Add a user with extension method approach
    let insert_user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    let rows_affected = conn.insert(insert_user)?;
    
    // Get a user with extension method approach
    let get_user = GetUser {
        id: 1,
        name: String::new(),
        email: String::new(),
    };
    let user = conn.fetch(&get_user)?;
    
    println!("User: {:?}", user);
    Ok(())
}
```

## Transaction Operations

parsql-sqlite offers two different approaches for transaction operations:

1. Using the `CrudOps` trait directly on the `Transaction` object
2. Using helper functions from the `transactional` module

### Transaction Operations with CrudOps Trait

```rust
use rusqlite::{Connection, Result};
use parsql::sqlite::CrudOps;
use parsql::sqlite::transactional;
use parsql::macros::{Insertable, SqlParams, Queryable, FromRow};

#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
}

#[derive(Queryable, FromRow, SqlParams)]
#[table("users")]
#[where_clause("id = ?")]
struct GetUser {
    id: i64,
    name: String,
    email: String,
}

fn main() -> Result<()> {
    let conn = Connection::open("test.db")?;
    
    // Begin a transaction
    let tx = transactional::begin(&conn)?;
    
    // Insert a user within the transaction
    let user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    let rows_affected = tx.insert(user)?;
    
    // Get a user within the transaction
    let param = GetUser {
        id: 1,
        name: String::new(),
        email: String::new(),
    };
    let user = tx.fetch(&param)?;
    
    // Commit the transaction
    tx.commit()?;
    
    println!("User: {:?}", user);
    Ok(())
}
```

### Transaction Operations with Transactional Module

```rust
use rusqlite::{Connection, Result};
use parsql::sqlite::transactional;
use parsql::macros::{Insertable, SqlParams, Updateable, UpdateParams};

#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
}

#[derive(Updateable, UpdateParams)]
#[table("users")]
#[update("email")]
#[where_clause("id = ?")]
struct UpdateUser {
    id: i64,
    email: String,
}

fn main() -> Result<()> {
    let conn = Connection::open("test.db")?;
    
    // Begin a transaction
    let tx = transactional::begin(&conn)?;
    
    // Insert a user within the transaction
    let insert_user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    let (tx, rows_affected) = transactional::tx_insert(tx, insert_user)?;
    
    // Update a user within the same transaction
    let update_user = UpdateUser {
        id: 1,
        email: "john.updated@example.com".to_string(),
    };
    let (tx, rows_affected) = transactional::tx_update(tx, update_user)?;
    
    // Commit the transaction - both operations succeed or fail together
    tx.commit()?;
    
    Ok(())
}
```

## CRUD Operations

### Reading Data (Get)

```rust
use parsql::{
    core::Queryable,
    macros::{FromRow, Queryable, SqlParams},
    sqlite::{FromRow, SqlParams, fetch},
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
    let get_result = fetch(&conn, get_user)?;
    
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
match fetch(&conn, get_user) {
    Ok(user) => println!("User found: {:?}", user),
    Err(e) => eprintln!("Error occurred: {}", e),
}
```

## Complete Example Project

For a complete example project, see the [examples/sqlite](../examples/sqlite) directory in the main parsql repository. 