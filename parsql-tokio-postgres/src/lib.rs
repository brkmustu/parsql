pub mod crud_ops;

pub use crud_ops::*;

use tokio_postgres::{types::ToSql, Row};

pub trait SqlParams {
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

pub trait UpdateParams {
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

pub trait FromRow {
    fn from_row(row: &Row) -> Self;
}

#[cfg(feature = "deadpool-postgres")]
pub mod transactional_ops;

#[cfg(feature = "deadpool-postgres")]
pub use transactional_ops as transactional;