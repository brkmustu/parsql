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

pub trait SqlQuery {
    fn query() -> String;
}
pub trait SqlParams {
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}
pub trait UpdateParams {
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}
pub trait FromRow {
    fn from_row(row: &Row) -> Result<Self, Error>
    where
        Self: Sized;
}

#[cfg(feature = "deadpool-postgres")]
pub mod transactional_ops;

#[cfg(feature = "deadpool-postgres")]
pub use transactional_ops as transactional;