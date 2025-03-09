//! # parsql-sqlite
//! 
//! SQLite integration for parsql.
//! This crate provides synchronous APIs for working with SQLite databases.
//! 
//! ## Features
//! 
//! - Synchronous SQLite operations
//! - Automatic SQL query generation
//! - Secure parameter management
//! - Generic CRUD operations
//! - Transaction support
//! 
//! ## Usage
//! 
//! ```rust,no_run
//! use rusqlite::{Connection, Result};
//! use parsql::sqlite::{get, insert};
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
//! #[where_clause("id = ?")]
//! pub struct GetUser {
//!     pub id: i32,
//!     pub name: String,
//!     pub email: String,
//! }
//! 
//! fn main() -> Result<()> {
//!     let conn = Connection::open("test.db")?;
//!     
//!     // Insert a new user
//!     let insert_user = InsertUser {
//!         name: "John".to_string(),
//!         email: "john@example.com".to_string(),
//!     };
//!     
//!     let id = insert(&conn, insert_user)?;
//!     
//!     // Get the user back
//!     let get_user = GetUser::new(id as i32);
//!     let user = get(&conn, &get_user)?;
//!     
//!     println!("User: {:?}", user);
//!     Ok(())
//! }
//! ```

pub mod crud_ops;

// Re-export sqlite types that might be needed
pub use rusqlite::{Connection, Error, Row};
pub use rusqlite::types::ToSql;

// Re-export crud operations
pub use crud_ops::{insert, select, select_all, update, delete, get, get_all};

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