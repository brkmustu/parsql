//! Prelude module for convenient imports
//! 
//! This module re-exports commonly used traits, macros and types to simplify usage of parsql.
//! 
//! # Example
//! 
//! ```rust,ignore
//! use parsql::prelude::*;
//! 
//! #[derive(Queryable)]
//! #[table("users")]
//! #[where_clause("id = $")]
//! pub struct GetUser {
//!     pub id: i64,
//!     pub name: String,
//! }
//! ```

// Priority order: deadpool-postgres > tokio-postgres > postgres > sqlite
// This ensures that when multiple features are enabled, we use the most advanced one

#[cfg(feature = "deadpool-postgres")]
pub use parsql_deadpool_postgres::macros::{FromRow, Queryable, SqlParams, Insertable, Updateable, Deletable, UpdateParams};

#[cfg(all(feature = "tokio-postgres", not(feature = "deadpool-postgres")))]
pub use parsql_tokio_postgres::macros::{FromRow, Queryable, SqlParams, Insertable, Updateable, Deletable, UpdateParams};

#[cfg(all(feature = "postgres", not(feature = "tokio-postgres"), not(feature = "deadpool-postgres")))]
pub use parsql_postgres::macros::{FromRow, Queryable, SqlParams, Insertable, Updateable, Deletable, UpdateParams};

#[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "tokio-postgres"), not(feature = "deadpool-postgres")))]
pub use parsql_sqlite::macros::{FromRow, Queryable, SqlParams, Insertable, Updateable, Deletable, UpdateParams};

// Re-export traits with both names - original for macros, and with Trait suffix for clarity
#[cfg(feature = "deadpool-postgres")]
pub use parsql_deadpool_postgres::traits::{
    CrudOps, 
    FromRow, FromRow as FromRowTrait, 
    SqlParams, SqlParams as SqlParamsTrait, 
    SqlQuery, 
    SqlCommand, 
    UpdateParams, UpdateParams as UpdateParamsTrait
};

#[cfg(all(feature = "tokio-postgres", not(feature = "deadpool-postgres")))]
pub use parsql_tokio_postgres::traits::{
    CrudOps, 
    FromRow, FromRow as FromRowTrait, 
    SqlParams, SqlParams as SqlParamsTrait, 
    SqlQuery, 
    SqlCommand, 
    UpdateParams, UpdateParams as UpdateParamsTrait
};

#[cfg(all(feature = "postgres", not(feature = "tokio-postgres"), not(feature = "deadpool-postgres")))]
pub use parsql_postgres::traits::{
    CrudOps, 
    FromRow, FromRow as FromRowTrait, 
    SqlParams, SqlParams as SqlParamsTrait, 
    SqlQuery, 
    SqlCommand, 
    UpdateParams, UpdateParams as UpdateParamsTrait
};

#[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "tokio-postgres"), not(feature = "deadpool-postgres")))]
pub use parsql_sqlite::traits::{
    CrudOps, 
    FromRow, FromRow as FromRowTrait, 
    SqlParams, SqlParams as SqlParamsTrait, 
    SqlQuery, 
    SqlCommand, 
    UpdateParams, UpdateParams as UpdateParamsTrait
};

// Re-export CRUD functions
#[cfg(feature = "deadpool-postgres")]
pub use parsql_deadpool_postgres::{insert, update, delete, fetch, fetch_all};

#[cfg(all(feature = "tokio-postgres", not(feature = "deadpool-postgres")))]
pub use parsql_tokio_postgres::{insert, update, delete, fetch, fetch_all};

#[cfg(all(feature = "postgres", not(feature = "tokio-postgres"), not(feature = "deadpool-postgres")))]
pub use parsql_postgres::{insert, update, delete, fetch, fetch_all};

#[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "tokio-postgres"), not(feature = "deadpool-postgres")))]
pub use parsql_sqlite::{insert, update, delete, fetch, fetch_all};

// Re-export database types - always include all enabled ones with different names
#[cfg(feature = "sqlite")]
pub use rusqlite::{Row as SqliteRow, ToSql as SqliteToSql, Error as SqliteError, Result as SqliteResult, Connection, params};

#[cfg(feature = "postgres")]
pub use postgres::{Row as PostgresRow, types::ToSql as PostgresToSql, Error as PostgresError, Client as PostgresClient, NoTls as PostgresNoTls, Transaction as PostgresTransaction};

#[cfg(any(feature = "tokio-postgres", feature = "deadpool-postgres"))]
pub use tokio_postgres::{Row as TokioPostgresRow, types::ToSql as TokioPostgresToSql, Error as TokioPostgresError, Client as TokioPostgresClient, NoTls as TokioPostgresNoTls, Transaction as TokioPostgresTransaction};

// For convenience, re-export the most commonly used types without prefixes based on active features
#[cfg(feature = "deadpool-postgres")]
pub use tokio_postgres::{Row, types::ToSql, Error, NoTls};

#[cfg(all(feature = "tokio-postgres", not(feature = "deadpool-postgres")))]
pub use tokio_postgres::{Row, types::ToSql, Error, Client, NoTls, Transaction};

#[cfg(all(feature = "postgres", not(feature = "tokio-postgres"), not(feature = "deadpool-postgres")))]
pub use postgres::{Row, types::ToSql, Error, Client, NoTls, Transaction};

#[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "tokio-postgres"), not(feature = "deadpool-postgres")))]
pub use rusqlite::{Row, ToSql, Error, Result, params as sql_params};