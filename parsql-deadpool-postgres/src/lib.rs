// Traits modülünü ekle
pub mod traits;
pub mod macros;

// Transaction işlemleri için modül
pub mod transactional_ops;

// Re-export macros
pub use macros::*;

// Transaction işlemleri için takma ad
pub use transactional_ops as transactional;

// CRUD işlemleri için modül
mod crud_ops;

// Pool extension işlemleri için modül
pub mod pool_extensions;
pub mod transaction_extensions;

// CRUD işlemlerini dışa aktar
pub use crud_ops::{
    insert,
    update,
    delete,
    fetch,
    fetch_all,
    get,
    get_all,
    select,
    select_all
};

// Deadpool-postgres türlerini dışa aktar
pub use deadpool_postgres::{Pool, Client as PoolClient, PoolError, Transaction};

// Public olarak Row ve Error türlerini dışa aktar
pub use tokio_postgres::{Error, Row};
pub use tokio_postgres::types::ToSql;