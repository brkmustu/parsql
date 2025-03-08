pub mod crud_ops;

pub use postgres::{Client, Error, Row};
pub use postgres::types::ToSql;

// Re-export crud operations
pub use crud_ops::{insert, select, select_all, update, delete, get, get_all, get_by_query};

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