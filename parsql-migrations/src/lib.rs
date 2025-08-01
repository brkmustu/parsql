//! # parsql-migrations
//!
//! Type-safe database migration system for parsql.
//!
//! This crate provides a flexible and safe way to manage database schema changes
//! across multiple database backends (PostgreSQL, SQLite, etc.).
//!
//! ## Features
//!
//! - Type-safe migration definitions
//! - Support for SQL and programmatic migrations
//! - Transaction support with automatic rollback
//! - Multi-database backend support
//! - Async runtime support
//! - Migration validation and checksums
//!
//! ## Example
//!
//! ```rust,no_run
//! use parsql_migrations::{Migration, MigrationConnection, MigrationError};
//!
//! struct CreateUsersTable;
//!
//! impl Migration for CreateUsersTable {
//!     fn version(&self) -> i64 { 1 }
//!     fn name(&self) -> &str { "create_users_table" }
//!     
//!     fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
//!         conn.execute(
//!             "CREATE TABLE users (
//!                 id BIGSERIAL PRIMARY KEY,
//!                 name VARCHAR(255) NOT NULL
//!             )"
//!         )
//!     }
//!     
//!     fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
//!         conn.execute("DROP TABLE IF EXISTS users")
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
// pub mod traits; // Temporarily disabled due to dyn compatibility issues
// pub mod traits_v2; // Temporarily disabled due to dyn compatibility issues
pub mod traits_simple;
pub mod types;
// pub mod runner; // Temporarily disabled due to dyn compatibility issues
// pub mod runner_v2; // Temporarily disabled due to dyn compatibility issues
pub mod runner_simple;
pub mod config;

// Feature-gated modules
#[cfg(feature = "postgres")]
pub mod postgres_simple;

#[cfg(feature = "sqlite")]
pub mod sqlite_simple;

// Async modules disabled temporarily
// #[cfg(feature = "tokio-postgres")]
// pub mod tokio_postgres;

// #[cfg(feature = "deadpool-postgres")]
// pub mod deadpool_postgres;

// Re-export commonly used types
pub use error::MigrationError;
pub use traits_simple::{Migration, MigrationConnection};
pub use types::{MigrationStatus, MigrationReport, MigrationDetails};
pub use runner_simple::MigrationRunner;
pub use config::MigrationConfig;

// Async traits disabled temporarily
// #[cfg(any(feature = "tokio-postgres", feature = "deadpool-postgres"))]
// pub use traits::AsyncMigrationConnection;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::error::MigrationError;
    pub use crate::traits_simple::{Migration, MigrationConnection};
    pub use crate::types::{MigrationStatus, MigrationReport};
    pub use crate::runner_simple::MigrationRunner;
    pub use crate::config::MigrationConfig;
    
    // Async traits disabled temporarily
    // #[cfg(any(feature = "tokio-postgres", feature = "deadpool-postgres"))]
    // pub use crate::traits::AsyncMigrationConnection;
}