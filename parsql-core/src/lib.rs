#[cfg(feature = "sqlite")]
pub use rusqlite::{types::ToSql, Row, Error};

#[cfg(feature = "postgres")]
pub use postgres::{types::ToSql, Row, Error};

#[cfg(any(
    feature = "tokio-postgres",
    feature = "deadpool-postgres"
))]
pub use tokio_postgres::{types::ToSql, Row, Error};

pub trait SqlQuery {
    fn query() -> String;
}

pub trait SqlParams {
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

pub trait UpdateParams {
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

#[cfg(feature = "sqlite")]
pub trait FromRow {
    fn from_row(row: &Row) -> Result<Self, Error>
    where
        Self: Sized;
}

#[cfg(any(
    feature = "postgres",
    feature = "tokio-postgres",
    feature = "deadpool-postgres"
))]
pub trait FromRow {
    fn from_row(row: &Row) -> Result<Self, Error>
    where
        Self: Sized;
}
