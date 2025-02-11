pub mod crud_ops;

// Re-export traits from parsql-core
pub use parsql_core::{SqlQuery, SqlParams, UpdateParams, FromRow};

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

#[cfg(feature = "deadpool-postgres")]
pub mod transactional_ops;

#[cfg(feature = "deadpool-postgres")]
pub use transactional_ops as transactional;