# parsql-postgres

PostgreSQL integration crate for parsql. This package provides **synchronous** APIs that enable parsql to work with PostgreSQL databases.

## Features

- Synchronous PostgreSQL operations
- Automatic SQL query generation
- Secure parameter management
- Generic CRUD operations (get, insert, update, delete)
- Conversion of database rows to structs

## Installation

Add to your Cargo.toml file as follows:

```toml
[dependencies]
parsql = { version = "0.3.0", features = ["postgres"] }
```

or if you want to use this package directly:

```toml
[dependencies]
parsql-postgres = "0.3.0"
parsql-macros = "0.3.0"
postgres = "0.19"
```

## Basic Usage

This package uses **synchronous operations** when working with PostgreSQL databases. This means it doesn't require async/await usage like tokio-postgres.

### Establishing a Connection

```rust
use postgres::{Client, NoTls, Error};

fn main() -> Result<(), Error> {
    // Create PostgreSQL connection
    let mut client = Client::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    )?;
    
    // Create example table
    client.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            state SMALLINT NOT NULL
        )",
        &[],
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
    postgres::{FromRow, SqlParams, get},
};
use postgres::{types::ToSql, Row};

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
    let mut client = Client::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    )?;
    
    // Usage
    let get_user = GetUser::new(1);
    let get_result = get(&mut client, get_user)?;
    
    println!("User: {:?}", get_result);
    Ok(())
}
```

### Adding Data (Insert)

```rust
use parsql::{
    core::Insertable,
    macros::{Insertable, SqlParams},
    postgres::{SqlParams, insert},
};
use postgres::types::ToSql;

#[derive(Insertable)]
#[table("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    )?;
    
    let insert_user = InsertUser {
        name: "Ali".to_string(),
        email: "ali@parsql.com".to_string(),
        state: 1,
    };
    
    let insert_result = insert(&mut client, insert_user)?;
    println!("Inserted record ID: {}", insert_result);
    
    Ok(())
}
```

### Updating Data (Update)

```rust
use parsql::{
    core::Updateable,
    macros::{UpdateParams, Updateable},
    postgres::{UpdateParams, update},
};
use postgres::types::ToSql;

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
    let mut client = Client::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    )?;
    
    let update_user = UpdateUser {
        id: 1,
        name: String::from("Ali Updated"),
        email: String::from("ali.updated@gmail.com"),
        state: 2,
    };
    
    let result = update(&mut client, update_user)?;
    println!("Number of records updated: {}", result);
    
    Ok(())
}
```

### Deleting Data (Delete)

```rust
use parsql::{
    core::Deletable,
    macros::{Deletable, SqlParams},
    postgres::{SqlParams, delete},
};
use postgres::types::ToSql;

#[derive(Deletable, SqlParams)]
#[table("users")]
#[where_clause("id = $")]
pub struct DeleteUser {
    pub id: i64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    )?;
    
    let delete_user = DeleteUser { id: 1 };
    let result = delete(&mut client, delete_user)?;
    
    println!("Number of records deleted: {}", result);
    Ok(())
}
```

## Using Transactions

You can use the Transaction feature provided by the standard postgres library:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    )?;
    
    // Start transaction
    let mut tx = client.transaction()?;
    
    // Perform queries within transaction
    let insert_user = InsertUser {
        name: "Mehmet".to_string(),
        email: "mehmet@parsql.com".to_string(),
        state: 1,
    };
    
    let id = insert(&mut tx, insert_user)?;
    
    // Complete the transaction
    tx.commit()?;
    
    println!("Insert operation successful, ID: {}", id);
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

This will print all queries generated for postgres to the console.

## Differences from Tokio-Postgres

The most important difference in this package is that it uses a **synchronous** API, unlike the tokio-postgres package:

1. **Synchronous Operations**: This package doesn't use async/await and performs synchronous operations.
2. **Client Reference**: You need to pass the client to functions as `&mut client`.
3. **No Tokio Runtime Required**: You can use it without needing a Tokio runtime.

## Performance Tips

1. **Prepared Statements**: postgres runs queries as prepared statements, and parsql uses this feature, which helps protect against SQL injection attacks.

2. **Batch Operations**: Perform multiple operations within a Transaction:

   ```rust
   let mut tx = client.transaction()?;
   // Perform your operations here
   tx.commit()?;
   ```

## Error Handling

Use Rust's `Result` mechanism to catch and handle errors that may occur during operations:

```rust
match get(&mut client, get_user) {
    Ok(user) => println!("User found: {:?}", user),
    Err(e) => eprintln!("Error occurred: {}", e),
}
```

## Complete Example Project

For a complete example project, see the [examples/postgres](../examples/postgres) directory in the main parsql repository. 