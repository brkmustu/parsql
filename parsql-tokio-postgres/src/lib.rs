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
//! - Connection pooling with deadpool-postgres
//! - Transaction support
//! 
//! ## Usage
//! 
//! ```rust,no_run
//! use tokio_postgres::{NoTls, Error};
//! use parsql::tokio_postgres::{get, insert};
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
//!     let id = insert(&client, insert_user).await?;
//!     
//!     // Get the user back
//!     let get_user = GetUser::new(id as i32);
//!     let user = get(&client, &get_user).await?;
//!     
//!     println!("User: {:?}", user);
//!     Ok(())
//! }
//! ```

pub mod crud_ops;

// Re-export tokio-postgres types that might be needed
pub use tokio_postgres::{types::ToSql, Row, Error, Client};

// Re-export crud operations
pub use crud_ops::{
    insert, 
    select, 
    select_all, 
    update, 
    delete, 
    get, 
    get_all
};

pub use parsql_macros as macros;

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

#[cfg(feature = "deadpool-postgres")]
pub mod transactional_ops;

#[cfg(feature = "deadpool-postgres")]
pub use transactional_ops as transactional;