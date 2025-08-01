use async_trait::async_trait;
use postgres::types::FromSql;
use std::fmt::Debug;
use tokio_postgres::types::ToSql;
use tokio_postgres::{Error, Row};

/// Trait for generating SQL queries (for SELECT operations).
/// This trait is implemented by the derive macro `Queryable`.
pub trait SqlQuery<R> {
    /// Returns the SQL query string.
    fn query() -> String;
}

/// Trait for generating SQL commands (for INSERT/UPDATE/DELETE operations).
/// This trait is implemented by the derive macros `Insertable`, `Updateable`, and `Deletable`.
pub trait SqlCommand {
    /// Returns the SQL command string.
    fn query() -> String;
}

/// Trait for providing SQL parameters.
/// This trait is implemented by the derive macro `SqlParams`.
pub trait SqlParams {
    /// Returns a vector of references to SQL parameters.
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

/// Trait for providing UPDATE parameters.
/// This trait is implemented by the derive macro `UpdateParams`.
pub trait UpdateParams {
    /// Returns a vector of references to SQL parameters for UPDATE operations.
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

/// Trait for converting database rows to Rust structs.
/// This trait is implemented by the derive macro `FromRow`.
pub trait FromRow {
    /// Converts a database row to a Rust struct.
    ///
    /// # Arguments
    /// * `row` - A reference to a database row
    ///
    /// # Returns
    /// * `Result<Self, Error>` - The converted struct or an error
    fn from_row(row: &Row) -> Result<Self, Error>
    where
        Self: Sized;
}

/// CrudOps trait'i, Pool nesnesi için CRUD işlemlerini extension method olarak sağlar.
/// Bu trait, Pool üzerinde doğrudan CRUD işlemlerini çağırmayı mümkün kılar.
#[async_trait]
pub trait CrudOps {
    /// Veritabanına yeni bir kayıt ekler.
    async fn insert<T, P: for<'a> FromSql<'a> + Send + Sync>(&self, entity: T) -> Result<P, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync;

    /// Veritabanındaki mevcut bir kaydı günceller.
    async fn update<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + UpdateParams + Send + Sync;

    /// Veritabanından bir kaydı siler.
    async fn delete<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync;

    /// Belirtilen kriterlere uygun tek bir kaydı getirir.
    async fn fetch<P, R>(&self, params: &P) -> Result<R, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync,
        R: FromRow + Send + Sync;

    /// Belirtilen kriterlere uygun tüm kayıtları getirir.
    async fn fetch_all<P, R>(&self, params: &P) -> Result<Vec<R>, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync,
        R: FromRow + Send + Sync;

    /// Belirtilen özel dönüşüm fonksiyonunu kullanarak tek bir kaydı getirir.
    async fn select<T, R, F>(&self, entity: T, to_model: F) -> Result<R, Error>
    where
        T: SqlQuery<T> + SqlParams + Send + Sync,
        F: FnOnce(&Row) -> Result<R, Error> + Send + Sync;

    /// Belirtilen özel dönüşüm fonksiyonunu kullanarak tüm kayıtları getirir.
    async fn select_all<T, R, F>(&self, entity: T, to_model: F) -> Result<Vec<R>, Error>
    where
        T: SqlQuery<T> + SqlParams + Send + Sync,
        F: Fn(&Row) -> R + Send + Sync;
}

/// TransactionOps trait, Transaction için CRUD işlemlerini extension method olarak sağlar
/// Bu şekilde, herhangi bir Transaction nesnesi üzerinde doğrudan CRUD işlemleri yapılabilir
#[async_trait]
pub trait TransactionOps {
    /// Insert method, yeni bir kayıt eklemek için kullanılır
    async fn tx_insert<T, P>(&self, entity: T) -> Result<P, Error>
    where
        T: SqlCommand + SqlParams + Debug + Send + 'static,
        P: for<'a> tokio_postgres::types::FromSql<'a> + Send + Sync;

    /// Update method, mevcut bir kaydı güncellemek için kullanılır
    async fn tx_update<T>(&self, entity: T) -> Result<bool, Error>
    where
        T: SqlCommand + UpdateParams + SqlParams + Debug + Send + 'static;

    /// Delete method, bir kaydı silmek için kullanılır
    async fn tx_delete<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + SqlParams + Debug + Send + 'static;

    /// Fetch method, tek bir kayıt getirmek için kullanılır
    async fn tx_fetch<P, R>(&self, params: &P) -> Result<R, Error>
    where
        P: SqlQuery<R> + SqlParams + Debug + Send + Sync + Clone + 'static,
        R: FromRow + Debug + Send + Sync + Clone + 'static;

    /// Fetch All method, birden fazla kayıt getirmek için kullanılır
    async fn tx_fetch_all<P, R>(&self, params: &P) -> Result<Vec<R>, Error>
    where
        P: SqlQuery<R> + SqlParams + Debug + Send + Sync + Clone + 'static,
        R: FromRow + Debug + Send + Sync + Clone + 'static;

    /// Select method, özel dönüşüm fonksiyonu ile tek bir kayıt getirmek için kullanılır
    async fn tx_select<T, F, R>(&self, entity: T, to_model: F) -> Result<R, Error>
    where
        T: SqlQuery<T> + SqlParams + Debug + Send + 'static,
        F: Fn(&Row) -> Result<R, Error> + Send + Sync + 'static,
        R: Send + 'static;

    /// Select All method, özel dönüşüm fonksiyonu ile birden fazla kayıt getirmek için kullanılır
    async fn tx_select_all<T, F, R>(&self, entity: T, to_model: F) -> Result<Vec<R>, Error>
    where
        T: SqlQuery<T> + SqlParams + Debug + Send + 'static,
        F: Fn(&Row) -> R + Send + Sync + 'static,
        R: Send + 'static;

    // Deprecated methods for backward compatibility
    #[deprecated(
        since = "0.2.0",
        note = "Renamed to `tx_insert`. Please use `tx_insert` function instead."
    )]
    async fn insert<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + SqlParams + Debug + Send + 'static;

    #[deprecated(
        since = "0.2.0",
        note = "Renamed to `tx_update`. Please use `tx_update` function instead."
    )]
    async fn update<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + UpdateParams + SqlParams + Debug + Send + 'static;

    #[deprecated(
        since = "0.2.0",
        note = "Renamed to `tx_delete`. Please use `tx_delete` function instead."
    )]
    async fn delete<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + SqlParams + Debug + Send + 'static;

    #[deprecated(
        since = "0.2.0",
        note = "Renamed to `tx_fetch`. Please use `tx_fetch` function instead."
    )]
    async fn get<T>(&self, params: &T) -> Result<T, Error>
    where
        T: SqlQuery<T> + FromRow + SqlParams + Debug + Send + Sync + Clone + 'static;

    #[deprecated(
        since = "0.2.0",
        note = "Renamed to `tx_fetch_all`. Please use `tx_fetch_all` function instead."
    )]
    async fn get_all<T>(&self, params: &T) -> Result<Vec<T>, Error>
    where
        T: SqlQuery<T> + FromRow + SqlParams + Debug + Send + Sync + Clone + 'static;

    #[deprecated(
        since = "0.2.0",
        note = "Renamed to `tx_select`. Please use `tx_select` function instead."
    )]
    async fn select<T, R, F>(&self, entity: T, to_model: F) -> Result<R, Error>
    where
        T: SqlQuery<T> + SqlParams + Debug + Send + 'static,
        F: for<'a> Fn(&'a Row) -> Result<R, Error> + Send + Sync + 'static,
        R: Send + 'static;

    #[deprecated(
        since = "0.2.0",
        note = "Renamed to `tx_select_all`. Please use `tx_select_all` function instead."
    )]
    async fn select_all<T, R, F>(&self, entity: T, to_model: F) -> Result<Vec<R>, Error>
    where
        T: SqlQuery<T> + SqlParams + Debug + Send + 'static,
        F: Fn(&Row) -> R + Send + Sync + 'static,
        R: Send + 'static;
}
