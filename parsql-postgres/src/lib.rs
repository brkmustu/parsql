pub mod crud_ops;

// Re-export traits from parsql-core
pub use parsql_core::{SqlQuery, SqlParams, UpdateParams, FromRow};

// Re-export postgres types that might be needed
pub use postgres::{Client, Error, Row};
pub use postgres::types::ToSql;

// Re-export crud operations
pub use crud_ops::{insert, select, select_all, update, delete, get, get_all, get_by_query};

pub use parsql_macros as macros;