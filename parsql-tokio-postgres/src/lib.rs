//! # parsql-tokio-postgres
//! 
//! Asynchronous PostgreSQL integration for parsql using tokio-postgres.
//! This crate provides async/await APIs for working with PostgreSQL databases.
//! 
//! ## Features
//! 
//! - Asynchronous PostgreSQL operations
//! - Automatic SQL query generation
//! - Secure parameter management
//! - Generic CRUD operations
//! - Deadpool connection pool support
//! - SQL Injection protection
//! - Detailed error reporting
//! 
//! ## Usage
//! 
//! ```rust,no_run
//! use tokio_postgres::{NoTls, Error};
//! use parsql::tokio_postgres::{CrudOps};
//! use parsql::macros::{Insertable, Queryable, SqlParams, FromRow};
//! 
//! #[derive(Insertable, SqlParams)]
//! #[table("users")]
//! pub struct InsertUser {
//!     pub name: String,
//!     pub email: String,
//! }
//! 
//! #[derive(Queryable, SqlParams, FromRow)]
//! #[table("users")]
//! #[where_clause("id = $")]
//! pub struct GetUser {
//!     pub id: i32,
//!     pub name: String,
//!     pub email: String,
//! }
//! 
//! impl GetUser {
//!     pub fn new(id: i32) -> Self {
//!         Self {
//!             id,
//!             name: Default::default(),
//!             email: Default::default(),
//!         }
//!     }
//! }
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     let (client, connection) = tokio_postgres::connect(
//!         "host=localhost user=postgres dbname=test",
//!         NoTls,
//!     ).await?;
//!     
//!     tokio::spawn(async move {
//!         if let Err(e) = connection.await {
//!             eprintln!("Connection error: {}", e);
//!         }
//!     });
//!     
//!     // Insert a new user
//!     let insert_user = InsertUser {
//!         name: "John".to_string(),
//!         email: "john@example.com".to_string(),
//!     };
//!     
//!     // Extension method style
//!     let id = client.insert(insert_user).await?;
//!     
//!     // Get the user back
//!     let get_user = GetUser::new(id as i32);
//!     let user = client.fetch(get_user).await?;
//!     
//!     println!("User: {:?}", user);
//!     Ok(())
//! }
//! ```
//!
//! The trait-based extension methods can also be used with client objects from deadpool-postgres:
//!
//! ```rust,no_run
//! use parsql::tokio_postgres::CrudOps;
//! use deadpool_postgres::{Config, Pool};
//! use tokio_postgres::NoTls;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut cfg = Config::new();
//!     cfg.host = Some("localhost".to_string());
//!     cfg.user = Some("postgres".to_string());
//!     
//!     let pool = cfg.create_pool(None, NoTls)?;
//!     
//!     // Get client from pool
//!     let client = pool.get().await?;
//!     
//!     // Use extension methods
//!     let users = client.fetch_all(active_users_query).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod crud_ops;

/// Transaction support module 
/// 
/// This module provides support for database transactions, including:
/// - Transaction management functions
/// - Implementation of `CrudOps` trait for `Transaction` objects
/// - Helper functions for working with transactions
///
/// There are two ways to use transactions:
/// 1. Using the `CrudOps` trait methods directly on a `Transaction` object
/// 2. Using the transaction helper functions from the `transactional` module
///
/// ```rust,no_run
/// use tokio_postgres::{NoTls, Error};
/// use parsql::tokio_postgres::{CrudOps, transactional};
/// use parsql::macros::{Insertable, SqlParams};
/// 
/// #[derive(Insertable, SqlParams)]
/// #[table("users")]
/// struct InsertUser {
///     name: String,
///     email: String,
/// }
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let (mut client, connection) = tokio_postgres::connect("", NoTls).await?;
///     tokio::spawn(async move { connection.await; });
///     
///     // Approach 1: CrudOps trait on Transaction
///     let tx = client.transaction().await?;
///     let rows = tx.insert(InsertUser {
///         name: "John".to_string(),
///         email: "john@example.com".to_string(),
///     }).await?;
///     tx.commit().await?;
///     
///     // Approach 2: Helper functions
///     let tx = transactional::begin(&mut client).await?;
///     let (tx, rows) = transactional::tx_insert(tx, InsertUser {
///         name: "Jane".to_string(),
///         email: "jane@example.com".to_string(),
///     }).await?;
///     tx.commit().await?;
///     
///     Ok(())
/// }
/// ```
pub mod transaction_ops;

// Re-export tokio-postgres types that might be needed
pub use tokio_postgres::{types::ToSql, Row, Error, Client};

// Re-export crud operations
pub use crate::crud_ops::{
    CrudOps,
    insert,
    update,
    delete,
    fetch,
    fetch_all,
    select,
    select_all
};

// Geriye dönük uyumluluk için eski fonksiyonları deprecated olarak dışa aktaralım
#[allow(deprecated)]
pub use crate::crud_ops::{
    get,
    get_all
};

pub use parsql_macros as macros;

/// Re-export transaction modules
/// 
/// This provides easy access to transaction functions via `transactional` namespace.
/// Functions include:
/// - `begin`: Begin a new transaction
/// - `tx_insert`: Insert a record within a transaction
/// - `tx_update`: Update records within a transaction
/// - `tx_delete`: Delete records within a transaction
/// - `tx_fetch`: Get a single record within a transaction  
/// - `tx_fetch_all`: Get multiple records within a transaction
/// - `tx_select`: Execute a custom query and transform a single result within a transaction
/// - `tx_select_all`: Execute a custom query and transform multiple results within a transaction
/// - `tx_get`: (Deprecated) Get a single record within a transaction
/// - `tx_get_all`: (Deprecated) Get multiple records within a transaction
pub use transaction_ops as transactional;

/// Trait for generating SQL queries.
/// This trait is implemented by the derive macro `Queryable`, `Insertable`, `Updateable`, and `Deletable`.
pub trait SqlQuery {
    /// Returns the SQL query string.
    fn query() -> String;
}

/// Trait for providing SQL parameters.
/// This trait is implemented by the derive macro `SqlParams`.
pub trait SqlParams {
    /// Returns a vector of references to SQL parameters.
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

/// Trait for providing UPDATE parameters.
/// This trait is implemented by the derive macro `UpdateParams`.
pub trait UpdateParams {
    /// Returns a vector of references to SQL parameters for UPDATE operations.
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

/// Trait for converting database rows to Rust structs.
/// This trait is implemented by the derive macro `FromRow`.
pub trait FromRow {
    /// Converts a database row to a Rust struct.
    /// 
    /// # Arguments
    /// * `row` - A reference to a database row
    /// 
    /// # Returns
    /// * `Result<Self, Error>` - The converted struct or an error
    fn from_row(row: &Row) -> Result<Self, Error>
    where
        Self: Sized;
}
