# parsql
An experimental SQL helper library

## What It Does

Parsql is a library that allows you to manage SQL queries directly through Rust structs. Its main purpose is to make database operations safer and with less code. With this library, you can:

- Generate automatic SQL queries from struct definitions
- Manage database parameters safely
- Easily perform generic CRUD operations (create, read, update, delete)
- Create dynamic SQL and execute complex queries

Parsql is not a standard ORM. Instead, it focuses on simplifying SQL writing and usage.

## Supported Databases

Parsql supports the following database systems:

- **SQLite** (synchronous): `parsql-sqlite` package
- **PostgreSQL** (synchronous): `parsql-postgres` package
- **Tokio PostgreSQL** (asynchronous): `parsql-tokio-postgres` package

## Installation

When adding the crate to your application, you need to specify which database you'll be working with as a 'feature'. You can add the package to your Cargo.toml file as follows:

### For SQLite
```rust
parsql = { version = "0.3.0", features = ["sqlite"] }
```

### For PostgreSQL
```rust
parsql = { version = "0.3.0", features = ["postgres"] }
```

### For Tokio PostgreSQL
```rust
parsql = { version = "0.3.0", features = ["tokio-postgres"] }
```

### For Deadpool PostgreSQL connection pool
```rust
parsql = { version = "0.3.0", features = ["deadpool-postgres"] }
```

## Core Features

### Procedural Macros
Parsql offers various procedural macros to facilitate database operations:

- `#[derive(Queryable)]` - For read (select) operations
- `#[derive(Insertable)]` - For insert operations
- `#[derive(Updateable)]` - For update operations
- `#[derive(FromRow)]` - For converting database results to objects

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

### SQL Tracing
To monitor SQL queries generated during development:

```sh
PARSQL_TRACE=1 cargo run
```

## Simple Usage Example

```rust
// To retrieve a record
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("id = $")]
pub struct GetUser {
    pub id: i64,
    pub name: String,
    pub email: String,
}

// For SQLite usage
let get_user = GetUser::new(1);
let user = get(&conn, get_user);

// For Tokio-Postgres usage
let get_user = GetUser::new(1);
let user = get(&client, get_user).await;
```

## Detailed Documentation

For more detailed information and examples for each database adapter, refer to the README files in the respective sub-packages:

- [SQLite Documentation](./parsql-sqlite/README.en.md)
- [PostgreSQL Documentation](./parsql-postgres/README.en.md)
- [Tokio PostgreSQL Documentation](./parsql-tokio-postgres/README.en.md)

You can find comprehensive example projects for each database type in the [examples folder](./examples) on GitHub.

## Changes in Version 0.3.0

- Added `join`, `group_by`, `order_by`, and `having` attributes
- Added `PARSQL_TRACE` environment variable support
- Updated attribute names (`table_name`→`table`, `update_clause`→`update`, `select_clause`→`select`)
- Added `SqlQuery` trait and simplified trait structure
- Moved core traits to the `parsql-core` crate 