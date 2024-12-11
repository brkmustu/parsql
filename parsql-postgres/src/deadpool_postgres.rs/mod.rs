pub mod transactional_ops;

pub use transactional_ops::*;

use tokio_postgres::{types::ToSql, Row, Error};

pub trait SqlParams {
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

pub trait DataMapper {
    fn to_model<R>(&self, row: &Row) -> Result<R, Error>;
}