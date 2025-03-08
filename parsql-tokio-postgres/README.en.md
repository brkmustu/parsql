# parsql-tokio-postgres

Tokio PostgreSQL integration crate for parsql. This package provides asynchronous APIs that enable parsql to work with tokio-postgres and deadpool-postgres libraries.

## Features

- Asynchronous PostgreSQL operations (with tokio runtime)
- Automatic SQL query generation
- Secure parameter management
- Generic CRUD operations (get, insert, update, delete)
- Conversion of database rows to structs
- Deadpool connection pool support

## Installation

Add to your Cargo.toml file as follows:

```toml
[dependencies]
# For Tokio PostgreSQL
parsql = { version = "0.3.0", features = ["tokio-postgres"] }

# or for using with deadpool connection pool
parsql = { version = "0.3.0", features = ["deadpool-postgres"] }
```

or if you want to use this package directly:

```toml
[dependencies]
parsql-tokio-postgres = "0.3.0"
parsql-macros = "0.3.0"
tokio-postgres = "0.7"
tokio = { version = "1", features = ["full"] }

# If you want to use Deadpool
deadpool-postgres = "0.10"
```

## Basic Usage

This package uses **asynchronous operations** when working with PostgreSQL databases. This means it requires async/await usage.

### Establishing a Connection

#### With Tokio PostgreSQL

```rust
use tokio_postgres::{NoTls, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create PostgreSQL connection
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    ).await?;
    
    // Run the connection in the background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    // Create example table
    client.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            state SMALLINT NOT NULL
        )",
        &[],
    ).await?;
    
    // ...
    
    Ok(())
}
```

#### With Deadpool PostgreSQL Connection Pool

```rust
use deadpool_postgres::{Config, Client, Pool};
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Deadpool configuration
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.user = Some("postgres".to_string());
    cfg.password = Some("postgres".to_string());
    cfg.dbname = Some("test".to_string());
    
    // Create connection pool
    let pool = cfg.create_pool(None, NoTls)?;
    
    // Get connection from pool
    let client: Client = pool.get().await?;
    
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
    tokio_postgres::{FromRow, SqlParams, get},
};
use tokio_postgres::{types::ToSql, Row};

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    // Usage
    let get_user = GetUser::new(1);
    let get_result = get(&client, get_user).await?;
    
    println!("User: {:?}", get_result);
    Ok(())
}
```

### Adding Data (Insert)

```rust
use parsql::{
    core::Insertable,
    macros::{Insertable, SqlParams},
    tokio_postgres::{SqlParams, insert},
};
use tokio_postgres::types::ToSql;

#[derive(Insertable)]
#[table("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    let insert_user = InsertUser {
        name: "Ali".to_string(),
        email: "ali@parsql.com".to_string(),
        state: 1,
    };
    
    let insert_result = insert(&client, insert_user).await?;
    println!("Inserted record ID: {}", insert_result);
    
    Ok(())
}
```

### Updating Data (Update)

```rust
use parsql::{
    core::Updateable,
    macros::{UpdateParams, Updateable},
    tokio_postgres::{UpdateParams, update},
};
use tokio_postgres::types::ToSql;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    let update_user = UpdateUser {
        id: 1,
        name: String::from("Ali Updated"),
        email: String::from("ali.updated@gmail.com"),
        state: 2,
    };
    
    let result = update(&client, update_user).await?;
    println!("Number of records updated: {}", result);
    
    Ok(())
}
```

### Deleting Data (Delete)

```rust
use parsql::{
    core::Deletable,
    macros::{Deletable, SqlParams},
    tokio_postgres::{SqlParams, delete},
};
use tokio_postgres::types::ToSql;

#[derive(Deletable, SqlParams)]
#[table("users")]
#[where_clause("id = $")]
pub struct DeleteUser {
    pub id: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    let delete_user = DeleteUser { id: 1 };
    let result = delete(&client, delete_user).await?;
    
    println!("Number of records deleted: {}", result);
    Ok(())
}
```

## Using Deadpool

To use parsql with Deadpool connection pool, first you need to enable the "deadpool-postgres" feature in your cargo.toml file. Then, you can use parsql functions on clients obtained from the pool.

```rust
use deadpool_postgres::{Config, Pool};
use tokio_postgres::NoTls;
use parsql::tokio_postgres::{get, insert};

// Get operation with pool connection
async fn fetch_user(pool: &Pool, user_id: i64) -> Result<GetUser, Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    let get_user = GetUser::new(user_id);
    let result = get(&client, get_user).await?;
    Ok(result)
}

// Insert operation with pool connection
async fn create_user(pool: &Pool, user: InsertUser) -> Result<i64, Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    let result = insert(&client, user).await?;
    Ok(result)
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

This will print all queries generated for tokio-postgres to the console.

## Performance Tips

1. **Prepared Statements**: tokio-postgres runs queries as prepared statements, and parsql uses this feature, which helps protect against SQL injection attacks.

2. **Connection Pool**: Using deadpool-postgres provides better performance for high-load applications.

3. **Asynchronous Operations**: By running your operations asynchronously with tokio-postgres, you can make your application more efficient.

## Error Handling

Use Rust's `Result` mechanism to catch and handle errors that may occur during tokio-postgres operations:

```rust
match get(&client, get_user).await {
    Ok(user) => println!("User found: {:?}", user),
    Err(e) => eprintln!("Error occurred: {}", e),
}
```

## Complete Example Project

For a complete example project, see the [examples/tokio-postgres](../examples/tokio-postgres) directory in the main parsql repository. 