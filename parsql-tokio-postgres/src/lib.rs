pub mod crud_ops;

pub use crud_ops::*;

use tokio_postgres::types::ToSql;

pub trait SqlParams {
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

#[cfg(feature = "deadpool-postgres")]
pub mod transactional_ops;

#[cfg(feature = "deadpool-postgres")]
pub use transactional_ops as transactional;