# parsql-tokio-postgres

Tokio PostgreSQL integration crate for parsql. This package provides asynchronous APIs that enable parsql to work with tokio-postgres and deadpool-postgres libraries.

## Features

- Asynchronous PostgreSQL operations (with tokio runtime)
- Automatic SQL query generation
- Secure parameter management
- Generic CRUD operations (fetch, insert, update, delete)
- Conversion of database rows to structs
- Deadpool connection pool support
- Automatic protection against SQL Injection attacks
- Detailed error reporting
- High-performance asynchronous query execution
- Extension methods (CrudOps trait)

## Security Features

### SQL Injection Protection

parsql-tokio-postgres is designed to be secure against SQL Injection attacks:

- All user inputs are automatically parameterized
- PostgreSQL's "$1, $2, ..." parameterization structure is used automatically
- Macros process SQL parameters securely, providing protection against injection attacks
- Parameters are automatically managed to ensure correct order and type
- User inputs in `#[where_clause]` and other SQL components are always parameterized
- Security measures are fully maintained even in asynchronous contexts

```rust
// SQL injection protection example
#[derive(Queryable, FromRow, SqlParams)]
#[table("users")]
#[where_clause("username = $ AND status = $")]
struct UserQuery {
    username: String,
    status: i32,
}

// User input (potentially harmful) is used safely
let query = UserQuery {
    username: user_input, // This value is not directly inserted into SQL, it's parameterized
    status: 1,
};

// Generated query: "SELECT * FROM users WHERE username = $1 AND status = $2"
// Parameters are safely sent as: [user_input, 1]
let user = fetch(&client, query).await?;
```

## Installation

Add to your Cargo.toml file as follows:

```toml
[dependencies]
# For Tokio PostgreSQL
parsql = { version = "0.3.2", features = ["tokio-postgres"] }

# or for using with deadpool connection pool
parsql = { version = "0.3.2", features = ["deadpool-postgres"] }
```

or if you want to use this package directly:

```toml
[dependencies]
parsql-tokio-postgres = "0.3.2"
parsql-macros = "0.3.2"
tokio-postgres = "0.7.13"
tokio = { version = "1.41.1", features = ["full"] }

# If you want to use Deadpool
deadpool-postgres = "0.14.1"
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

There are two approaches provided for performing database operations:

1. Function-based approach (`fetch`, `insert`, `update`, etc.)
2. Extension method approach (`client.fetch()`, `client.insert()`, etc.) - `CrudOps` trait

### Extension Method Approach (CrudOps Trait)

```rust
use parsql::{
    macros::{Queryable, FromRow, SqlParams},
    tokio_postgres::{CrudOps},
};

#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("id = $")]
pub struct GetUser {
    pub id: i64,
    pub name: String,
    pub email: String,
}

impl GetUser {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            name: Default::default(),
            email: Default::default(),
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
    
    // Using extension method
    let get_user = GetUser::new(1);
    let user = client.fetch(get_user).await?;
    
    println!("User: {:?}", user);
    Ok(())
}
```

### Reading Data (Fetch)

```rust
use parsql::{
    core::Queryable,
    macros::{FromRow, Queryable, SqlParams},
    tokio_postgres::{FromRow, SqlParams, fetch, CrudOps},
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
    
    // Usage (function approach)
    let get_user = GetUser::new(1);
    let get_result = fetch(&client, get_user).await?;
    
    // or (extension method approach)
    let get_user = GetUser::new(1);
    let get_result = client.fetch(get_user).await?;
    
    println!("User: {:?}", get_result);
    Ok(())
}
```

### Adding Data (Insert)

```rust
use parsql::{
    core::Insertable,
    macros::{Insertable, SqlParams},
    tokio_postgres::{SqlParams, insert, CrudOps},
};
use tokio_postgres::types::ToSql;

#[derive(Insertable, SqlParams)]
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
        name: "John".to_string(),
        email: "john@parsql.com".to_string(),
        state: 1,
    };
    
    // Function approach
    let insert_result = insert(&client, insert_user).await?;
    
    // or extension method approach
    let insert_user = InsertUser {
        name: "Alice".to_string(),
        email: "alice@parsql.com".to_string(),
        state: 1,
    };
    let insert_result = client.insert(insert_user).await?;
    
    println!("Inserted record ID: {}", insert_result);
    
    Ok(())
}
```

### Updating Data (Update)

```rust
use parsql::{
    core::Updateable,
    macros::{UpdateParams, Updateable},
    tokio_postgres::{UpdateParams, update, CrudOps},
};

#[derive(Updateable, UpdateParams)]
#[table("users")]
#[update("name, email")]
#[where_clause("id = $")]
pub struct UpdateUser {
    pub id: i64,
    pub name: String,
    pub email: String,
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
        name: "John Smith".to_string(),
        email: "john.smith@parsql.com".to_string(),
    };
    
    // Function approach
    let update_result = update(&client, update_user).await?;
    
    // or extension method approach
    let update_user = UpdateUser {
        id: 2,
        name: "Alice Johnson".to_string(),
        email: "alice.johnson@parsql.com".to_string(),
    };
    let update_result = client.update(update_user).await?;
    
    println!("Update successful: {}", update_result);
    
    Ok(())
}
```

### Deleting Data (Delete)

```rust
use parsql::{
    core::Deletable,
    macros::{Deletable, SqlParams},
    tokio_postgres::{SqlParams, delete, CrudOps},
};

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
    
    // Function approach
    let delete_result = delete(&client, delete_user).await?;
    
    // or extension method approach
    let delete_user = DeleteUser { id: 2 };
    let delete_result = client.delete(delete_user).await?;
    
    println!("Number of records deleted: {}", delete_result);
    
    Ok(())
}
```

## Transaction Operations

There are two different approaches for performing database operations within a transaction:

#### 1. Using the `CrudOps` trait on the Transaction object

The `Transaction` struct implements the `CrudOps` trait, allowing you to use the same extension methods on the `Transaction` object as you would on a `Client` object:

```rust
use tokio_postgres::{NoTls, Error};
use parsql::tokio_postgres::{CrudOps, transactional};
use parsql::macros::{Insertable, Updateable, SqlParams};

#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
    state: i16,
}

#[derive(Updateable, SqlParams)]
#[table("users")]
#[where_clause("id = $")]
struct ActivateUser {
    id: i64,
    state: i16,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (mut client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    // Start a transaction
    let transaction = client.transaction().await?;
    
    // Insert a user using CrudOps trait method
    let user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
        state: 0, // inactive
    };
    
    // Direct insert operation within transaction
    let rows = transaction.insert(user).await?;
    println!("Rows affected: {}", rows);
    
    // Activate the user using CrudOps trait method
    let activate = ActivateUser {
        id: 1, // ID of the inserted user
        state: 1, // active
    };
    
    // Direct update operation within transaction
    let updated = transaction.update(activate).await?;
    println!("Update successful: {}", updated);
    
    // Commit the transaction
    transaction.commit().await?;
    
    Ok(())
}
```

#### 2. Using the `transactional` module

You can use the helper functions from the `transactional` module to perform operations:

```rust
use tokio_postgres::{NoTls, Error};
use parsql::tokio_postgres::transactional;
use parsql::macros::{Insertable, Updateable, SqlParams};

#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
    state: i16,
}

#[derive(Updateable, SqlParams)]
#[table("users")]
#[where_clause("id = $")]
struct ActivateUser {
    id: i64,
    state: i16,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (mut client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    // Start a transaction
    let tx = transactional::begin(&mut client).await?;
    
    // Insert a user
    let user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
        state: 0, // inactive
    };
    
    // Insert operation within transaction
    let (tx, rows) = transactional::tx_insert(tx, user).await?;
    println!("Rows affected: {}", rows);
    
    // Activate the user
    let activate = ActivateUser {
        id: 1, // ID of the inserted user
        state: 1, // active
    };
    
    // Update operation within transaction
    let (tx, updated) = transactional::tx_update(tx, activate).await?;
    println!("Update successful: {}", updated);
    
    // Commit the transaction
    tx.commit().await?;
    
    Ok(())
}
```

### Transaction Helper Functions

The `transactional` module provides the following functions:

- `begin(&mut client)`: Starts a new transaction
- `tx_insert(transaction, entity)`: Performs an insert operation within a transaction and returns the transaction and the number of affected rows
- `tx_update(transaction, entity)`: Performs an update operation within a transaction and returns the transaction and the update status
- `tx_delete(transaction, entity)`: Performs a delete operation within a transaction and returns the transaction and the number of deleted rows
- `tx_get(transaction, params)`: Retrieves a single record within a transaction and returns the transaction and the record
- `tx_get_all(transaction, params)`: Retrieves multiple records within a transaction and returns the transaction and the records
- `tx_select(transaction, entity, to_model)`: Executes a custom query within a transaction and transforms the result using a provided function
- `tx_select_all(transaction, entity, to_model)`: Executes a custom query within a transaction and transforms all results using a provided function

Each function returns the transaction object, allowing you to chain operations:

```rust
let tx = transactional::begin(&mut client).await?;
let (tx, _) = transactional::tx_insert(tx, user1).await?;
let (tx, _) = transactional::tx_insert(tx, user2).await?;
let (tx, _) = transactional::tx_update(tx, activate_user).await?;
tx.commit().await?;
```

## Performance Tips

* Reuse queries with the same SQL structure to take advantage of the query plan cache
* Use connection pools for large numbers of queries
* Use pagination (limit and offset) instead of `get_all` for large datasets
* Apply filters at the database level, not in your application

## Error Catching and Handling

```rust
async fn handle_database() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    let result = match fetch(&client, user_query).await {
        Ok(user) => {
            println!("User found: {:?}", user);
            // Operation successful
            Ok(())
        },
        Err(e) => match e.code() {
            // Handle specific PostgreSQL error codes
            Some(code) if code == &tokio_postgres::error::SqlState::UNIQUE_VIOLATION => {
                println!("Uniqueness violation: {}", e);
                Err(e.into())
            },
            Some(code) if code == &tokio_postgres::error::SqlState::FOREIGN_KEY_VIOLATION => {
                println!("Foreign key violation: {}", e);
                Err(e.into())
            },
            _ => {
                println!("General database error: {}", e);
                Err(e.into())
            }
        },
    };
    
    result
}
```

## Licensing

This library is licensed under the MIT or Apache-2.0 license. 