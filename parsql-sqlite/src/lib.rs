pub mod crud_ops;

// Re-export traits from parsql-core
pub use parsql_core::{SqlQuery, SqlParams, UpdateParams, FromRow};

// Re-export sqlite types that might be needed
pub use rusqlite::{Connection, Error, Row};
pub use rusqlite::types::ToSql;

// Re-export crud operations
pub use crud_ops::{insert, select, select_all, update, delete, get, get_all};

pub use parsql_macros as macros;