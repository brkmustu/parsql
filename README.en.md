# parsql

[![Version](https://img.shields.io/crates/v/parsql.svg)](https://crates.io/crates/parsql)
[![Documentation](https://docs.rs/parsql/badge.svg)](https://docs.rs/parsql)
[![License](https://img.shields.io/crates/l/parsql.svg)](https://github.com/yazdostum-nettr/parsql/blob/master/LICENSE)

An experimental SQL helper crate. This is not an ORM tool. The aim is to make it easier to write and use simple SQL statements.

## Features

- Automatic SQL query generation
- Secure parameter management
- Support for multiple database systems (PostgreSQL, SQLite, Tokio PostgreSQL, Deadpool PostgreSQL)
- Type-safe database operations
- Automatic protection against SQL Injection attacks
- **New (0.3.3):** Complete pagination support: efficiently paginate results using `limit` and `offset` attributes
- `Queryable` derivation attribute supports table name, where statement, select statement, group by, having, order by, limit and offset statements.
- `Insertable` derivation attribute generates table-specific INSERT statements.
- `Updateable` derivation attribute generates table-specific UPDATE statements.
- `Deletable` derivation attribute generates table-specific DELETE statements.
- `SqlParams` derivation attribute allows the structure to be used for SQL parameters.
- `UpdateParams` derivation attribute allows the structure to be used for UPDATE statements.
- `FromRow` derivation attribute allows database rows to be converted to the structure.
- **New (0.3.3):** Added `PARSQL_TRACE` environment variable support for SQL trace logging.

## What It Does

Parsql is a library that allows you to manage SQL queries directly through Rust structs. Its main purpose is to make database operations safer and with less code. With this library, you can:

- Generate automatic SQL queries from struct definitions
- Manage database parameters safely
- Easily perform generic CRUD operations (create, read, update, delete)
- Create dynamic SQL and execute complex queries
- Easily perform asynchronous database operations
- Get automatic protection against SQL injection attacks
- Use extension methods directly on `Pool` and `Transaction` objects

Parsql is not a standard ORM. Instead, it focuses on simplifying SQL writing and usage.

## Supported Databases

Parsql supports the following database systems:

- **SQLite** (synchronous): `parsql-sqlite` package
- **PostgreSQL** (synchronous): `parsql-postgres` package
- **Tokio PostgreSQL** (asynchronous): `parsql-tokio-postgres` package
- **Deadpool PostgreSQL** (asynchronous connection pool): `parsql-deadpool-postgres` package

## Installation

Add to your Cargo.toml file as follows:

```toml
[dependencies]
parsql = "0.3.6"
```

and select the feature you need:

```toml
# For SQLite
parsql = { version = "0.3.6", features = ["sqlite"] }

# For PostgreSQL
parsql = { version = "0.3.6", features = ["postgres"] }

# For Tokio PostgreSQL
parsql = { version = "0.3.6", features = ["tokio-postgres"] }

# For Deadpool PostgreSQL
parsql = { version = "0.3.6", features = ["deadpool-postgres"] }
```

## Core Features

### Procedural Macros
Parsql offers various procedural macros to facilitate database operations:

- `#[derive(Queryable)]` - For read (select) operations
- `#[derive(Insertable)]` - For insert operations
- `#[derive(Updateable)]` - For update operations
- `#[derive(Deletable)]` - For delete operations
- `#[derive(FromRow)]` - For converting database results to objects
- `#[derive(SqlParams)]` - For configuring SQL parameters
- `#[derive(UpdateParams)]` - For configuring update parameters

### Extension Method Usage

Since version 0.3.3, Parsql provides extension methods that allow you to perform CRUD operations directly on database objects. This approach makes your code more fluid and readable.

#### Extension Methods on Pool Objects

You can perform CRUD operations directly on connection pool (Pool) objects:

```rust
// Traditional usage
let rows_affected = insert(&pool, user).await?;

// Using extension method
use parsql_deadpool_postgres::CrudOps;
let rows_affected = pool.insert(user).await?;
```

#### Extension Methods on Transaction Objects

You can perform CRUD operations directly on Transaction objects:

```rust
// Traditional usage
let (tx, rows_affected) = tx_insert(tx, user).await?;

// Using extension method
use parsql_deadpool_postgres::TransactionOps;
let rows_affected = tx.insert(user).await?;
```

#### Supported Extension Methods

The following extension methods are available for both Pool and Transaction objects:

- `insert(entity)` - Inserts a record
- `update(entity)` - Updates a record
- `delete(entity)` - Deletes a record
- `fetch(params)` - Retrieves a single record
- `fetch_all(params)` - Retrieves multiple records
- `select(entity, to_model)` - Retrieves a single record with a custom transformer function
- `select_all(entity, to_model)` - Retrieves multiple records with a custom transformer function

### Transaction Support

Parsql currently provides transaction support in the following packages:

- `parsql-postgres` - Transaction support for synchronous PostgreSQL operations
- `parsql-tokio-postgres` - Transaction support for asynchronous Tokio-PostgreSQL operations
- `parsql-deadpool-postgres` - Transaction support for asynchronous Deadpool PostgreSQL connection pool

Example of transaction usage:

```rust
// Start a transaction
let client = pool.get().await?;
let tx = client.transaction().await?;

// Perform operations within the transaction using extension methods
let result = tx.insert(user).await?;
let rows_affected = tx.update(user_update).await?;

// Commit if successful
tx.commit().await?;
```

### Security Features

#### SQL Injection Protection
Parsql is designed to be secure against SQL injection attacks:

- Parameterized queries are automatically used, never direct string concatenation
- All user inputs are safely parameterized
- Macros process SQL parameters correctly and provide a secure format
- Appropriate parameter placeholders (`$1`, `?`, etc.) are automatically applied for each database adapter
- The need for manual string concatenation when writing SQL is eliminated
- Security measures are fully maintained even in asynchronous contexts

```rust
// Example of secure parameter usage
#[derive(Queryable, FromRow, SqlParams)]
#[table("users")]
#[where_clause("username = $ AND status = $")]
struct UserQuery {
    username: String,
    status: i32,
}

// Parameters are securely placed,
// no risk of SQL injection
let query = UserQuery {
    username: user_input,
    status: 1,
};
```

### Attributes
You can use various attributes to customize your queries:

- `#[table("table_name")]` - To specify the table name
- `#[where_clause("id = $")]` - To specify the WHERE condition
- `#[select("field1, field2")]` - To customize the SELECT statement
- `#[update("field1, field2")]` - To customize the UPDATE statement
- `#[join("LEFT JOIN table2 ON table1.id = table2.fk_id")]` - For JOIN statements
- `#[group_by("field1")]` - For GROUP BY statements
- `#[order_by("field1 DESC")]` - For ORDER BY statements
- `#[having("COUNT(*) > 5")]` - For HAVING statements
- `#[limit(10)]` - For LIMIT statements
- `#[offset(5)]` - For OFFSET statements
- `#[returning("id")]` - To specify returning values from INSERT/UPDATE operations

### SQL Tracing
To monitor SQL queries generated during development:

```sh
PARSQL_TRACE=1 cargo run
```

This will print all executed SQL queries to the console.

## Simple Usage Examples

### Using with SQLite

```rust
use parsql::{
    sqlite::{fetch, insert},
    macros::{Queryable, Insertable, FromRow, SqlParams},
};
use rusqlite::Connection;

// To retrieve a record
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

// To insert a new record
#[derive(Insertable, SqlParams)]
#[table("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("test.db")?;
    
    let insert_user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    
    let id = insert(&conn, insert_user)?;
    println!("Inserted record ID: {}", id);
    
    let get_user = GetUser::new(id);
    let user = fetch(&conn, get_user)?;
    println!("User: {:?}", user);
    
    Ok(())
}
```

### Using Deadpool PostgreSQL with Async Connection Pool

```rust
use parsql_deadpool_postgres::{CrudOps, TransactionOps};
use tokio_postgres::NoTls;
use deadpool_postgres::{Config, Runtime};
use parsql_macros::{Queryable, Insertable, FromRow, SqlParams, Updateable};

#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("id = $")]
pub struct GetUser {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[derive(Insertable, SqlParams)]
#[table("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
}

#[derive(Updateable, SqlParams)]
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
    // Create connection pool
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.user = Some("postgres".to_string());
    cfg.password = Some("postgres".to_string());
    cfg.dbname = Some("test".to_string());
    
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
    
    // Insert using extension method
    let insert_user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    let rows_affected = pool.insert(insert_user).await?;
    println!("Number of inserted records: {}", rows_affected);
    
    // Using transaction
    let client = pool.get().await?;
    let tx = client.transaction().await?;
    
    // Update within transaction using extension method
    let update_user = UpdateUser {
        id: 1,
        name: "John Updated".to_string(),
        email: "john.updated@example.com".to_string(),
    };
    let rows_affected = tx.update(update_user).await?;
    
    // Commit if successful
    tx.commit().await?;
    
    Ok(())
}
```

## Performance Tips

- Reuse queries with the same SQL structure to take advantage of the query plan cache
- Use connection pools for database-intensive applications
- Use pagination (limit and offset) instead of `get_all` for large datasets
- Apply filters at the database level, not in your application

## Detailed Documentation

For more detailed information and examples for each database adapter, refer to the README files in the respective sub-packages:

- [SQLite Documentation](./parsql-sqlite/README.en.md)
- [PostgreSQL Documentation](./parsql-postgres/README.en.md)
- [Tokio PostgreSQL Documentation](./parsql-tokio-postgres/README.en.md)
- [Deadpool PostgreSQL Documentation](./parsql-deadpool-postgres/README.en.md)

## License

This project is licensed under the MIT License.

Features:
- Automatic SQL query generation
- Parameter management with type safety
- Support for multiple database systems (PostgreSQL, SQLite)
- SQL injection protection with macros
- Pagination with Limit and Offset support

### Queryable

This derivation macro adds the ability to create SELECT queries to a structure.

Supported attributes:
- `table`: SQL table name
- `where_clause`: SQL WHERE statement
- `select`: SQL SELECT statement 
- `group_by`: SQL GROUP BY statement
- `having`: SQL HAVING statement
- `order_by`: SQL ORDER BY statement
- `limit`: SQL LIMIT statement
- `offset`: SQL OFFSET statement

### Reading Data (Fetch)

```rust
use parsql::{
    core::Queryable,
    macros::{FromRow, Queryable, SqlParams},
    postgres::{FromRow, SqlParams, fetch},
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
    let mut client = Client::connect("database.db", NoTls)?;
    
    // Usage
    let get_user = GetUser::new(1);
    let get_result = fetch(&mut client, &get_user)?;
    
    println!("User: {:?}", get_result);
    Ok(())
}
```
