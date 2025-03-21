# parsql-postgres

PostgreSQL integration crate for parsql. This package provides **synchronous** APIs that enable parsql to work with PostgreSQL databases.

## Features

- Synchronous PostgreSQL operations
- Automatic SQL query generation
- Secure parameter management
- Generic CRUD operations (get, insert, update, delete)
- Extension methods for the Client object
- Transaction support
- Conversion of database rows to structs
- Automatic protection against SQL Injection attacks

## Security Features

### SQL Injection Protection

parsql-postgres is designed to be secure against SQL Injection attacks:

- All user inputs are automatically parameterized
- PostgreSQL's "$1, $2, ..." parameterization structure is used automatically
- Macros process SQL parameters securely, providing protection against injection attacks
- Parameters are automatically managed to ensure correct order and type
- User inputs in `#[where_clause]` and other SQL components are always parameterized

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
let user = get(&conn, query)?;
```

## Installation

Add to your Cargo.toml file as follows:

```toml
[dependencies]
parsql = { version = "0.3.7", features = ["postgres"] }
```

or if you want to use this package directly:

```toml
[dependencies]
parsql-postgres = "0.3.2"
parsql-macros = "0.3.2"
postgres = "0.19"
```

## Usage

You can work with parsql-postgres using two different approaches:

### 1. Function-Based Approach

```rust
use postgres::{Client, NoTls};
use parsql::postgres::{get, insert};
use parsql::macros::{Insertable, SqlParams, Queryable, FromRow};

#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
}

#[derive(Queryable, FromRow, SqlParams)]
#[table("users")]
#[where_clause("id = $1")]
struct GetUser {
    id: i32,
    name: String,
    email: String,
}

fn main() -> Result<(), postgres::Error> {
    let mut client = Client::connect("host=localhost user=postgres", NoTls)?;
    
    // Adding a user with the function approach
    let insert_user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    let rows_affected = insert(&mut client, insert_user)?;
    
    // Getting a user with the function approach
    let get_user = GetUser {
        id: 1,
        name: String::new(),
        email: String::new(),
    };
    let user = get(&mut client, &get_user)?;
    
    println!("User: {:?}", user);
    Ok(())
}
```

### 2. Extension Method Approach (CrudOps Trait)

With this approach, you can call CRUD operations directly on the `Client` object thanks to the `CrudOps` trait:

```rust
use postgres::{Client, NoTls};
use parsql::postgres::CrudOps;  // Import the CrudOps trait
use parsql::macros::{Insertable, SqlParams, Queryable, FromRow};

#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
}

#[derive(Queryable, FromRow, SqlParams)]
#[table("users")]
#[where_clause("id = $1")]
struct GetUser {
    id: i32,
    name: String,
    email: String,
}

fn main() -> Result<(), postgres::Error> {
    let mut client = Client::connect("host=localhost user=postgres", NoTls)?;
    
    // Adding a user with the extension method approach
    let insert_user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    let rows_affected = client.insert(insert_user)?;
    
    // Getting a user with the extension method approach
    let get_user = GetUser {
        id: 1,
        name: String::new(),
        email: String::new(),
    };
    let user = client.get(&get_user)?;
    
    println!("User: {:?}", user);
    Ok(())
}
```

The extension method approach makes your code more readable and fluid, especially in situations where multiple CRUD operations are performed at the same time.

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

parsql-postgres supports transactions in two different ways:

### 1. Using CrudOps Trait with Transaction

With this approach, the `CrudOps` trait is implemented for the `Transaction` struct, allowing you to perform CRUD operations directly on the transaction object:

```rust
use postgres::{Client, NoTls};
use parsql::postgres::CrudOps;  // Import the CrudOps trait
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
#[where_clause("id = $")]
struct UpdateUser {
    id: i32,
    email: String,
}

fn main() -> Result<(), postgres::Error> {
    let mut client = Client::connect("host=localhost user=postgres", NoTls)?;
    
    // Start a transaction
    let mut tx = client.transaction()?;
    
    // Use CrudOps methods directly on the transaction
    let insert_user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    let rows_affected = tx.insert(insert_user)?;
    
    let update_user = UpdateUser {
        id: 1,
        email: "john.updated@example.com".to_string(),
    };
    let rows_updated = tx.update(update_user)?;
    
    // Commit the transaction
    tx.commit()?;
    Ok(())
}
```

### 2. Using Transaction Helper Functions

With this approach, you can use helper functions from the `transactional` module to perform operations with method chaining:

```rust
use postgres::{Client, NoTls};
use parsql::postgres::transactional::{begin, tx_insert, tx_update};
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
#[where_clause("id = $")]
struct UpdateUser {
    id: i32,
    email: String,
}

fn main() -> Result<(), postgres::Error> {
    let mut client = Client::connect("host=localhost user=postgres", NoTls)?;
    
    // Begin a transaction
    let tx = begin(&mut client)?;
    
    // Chain transaction operations
    let insert_user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    
    let (tx, _) = tx_insert(tx, insert_user)?;
    
    let update_user = UpdateUser {
        id: 1,
        email: "john.updated@example.com".to_string(),
    };
    
    let (tx, _) = tx_update(tx, update_user)?;
    
    // Commit the transaction
    tx.commit()?;
    Ok(())
}
```

This approach enhances code readability, especially when performing multiple operations within a transaction, as it ensures the transaction object is continuously available through the chain of operations.

## Complete Example with Transactions

Here's a more comprehensive example demonstrating transaction usage:

```rust
use postgres::{Client, NoTls, Error};
use parsql::postgres::CrudOps;
use parsql::postgres::transactional::{begin, tx_insert, tx_update};
use parsql::macros::{Insertable, SqlParams, Updateable, UpdateParams, Queryable, FromRow};

#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
    status: i16,
}

#[derive(Updateable, UpdateParams)]
#[table("users")]
#[update("status")]
#[where_clause("id = $")]
struct UpdateUserStatus {
    id: i32,
    status: i16,
}

#[derive(Queryable, FromRow, SqlParams)]
#[table("users")]
#[where_clause("id = $")]
struct GetUser {
    id: i32,
    name: String,
    email: String,
    status: i16,
}

fn main() -> Result<(), Error> {
    let mut client = Client::connect(
        "host=localhost user=postgres dbname=test",
        NoTls,
    )?;
    
    // Create table if not exists
    client.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            status SMALLINT NOT NULL
        )",
        &[],
    )?;
    
    // Method 1: Using CrudOps trait with Transaction
    {
        let mut tx = client.transaction()?;
        
        let insert_user = InsertUser {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            status: 1,
        };
        
        tx.insert(insert_user)?;
        
        let update_status = UpdateUserStatus {
            id: 1,
            status: 2,
        };
        
        tx.update(update_status)?;
        
        // Commit the transaction
        tx.commit()?;
    }
    
    // Method 2: Using transactional helper functions
    {
        let tx = begin(&mut client)?;
        
        let insert_user = InsertUser {
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            status: 1,
        };
        
        let (tx, _) = tx_insert(tx, insert_user)?;
        
        let update_status = UpdateUserStatus {
            id: 2,
            status: 3,
        };
        
        let (tx, _) = tx_update(tx, update_status)?;
        
        // Commit the transaction
        tx.commit()?;
    }
    
    // Verify the results
    let get_alice = GetUser {
        id: 1,
        name: String::new(),
        email: String::new(),
        status: 0,
    };
    
    let alice = client.get(&get_alice)?;
    println!("Alice: {:?}", alice);
    
    let get_bob = GetUser {
        id: 2,
        name: String::new(),
        email: String::new(),
        status: 0,
    };
    
    let bob = client.get(&get_bob)?;
    println!("Bob: {:?}", bob);
    
    Ok(())
}
```

This example shows both transaction approaches and how they can be used for complex database operations with multiple steps that need to be atomic.

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