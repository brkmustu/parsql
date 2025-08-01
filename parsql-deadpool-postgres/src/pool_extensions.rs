use crate::traits::{CrudOps, FromRow, SqlCommand, SqlParams, SqlQuery, UpdateParams};
use deadpool_postgres::{GenericClient, Pool};
use postgres::types::FromSql;
use std::sync::OnceLock;
use tokio_postgres::{Error, Row};

// Daha basit bir yaklaşım: PoolError'dan genel bir Error oluştur
fn pool_err_to_io_err(e: deadpool_postgres::PoolError) -> Error {
    // Bu özel fonksiyon tokio_postgres'in sağladığı timeout hatasını döndürür
    // Güzel bir çözüm değil, ama çalışır bir örnek için kullanılabilir
    let err = Error::__private_api_timeout();

    // Debug süreci için stderr'e hatayı yazdıralım
    eprintln!("Pool bağlantı hatası: {}", e);

    err
}

/// Pool extension trait for additional query operations
#[async_trait::async_trait]
pub trait PoolExtensions {
    /// Inserts a new record into the database
    async fn insert<T, P: for<'a> FromSql<'a> + Send + Sync>(&self, entity: T) -> Result<P, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync + 'static;

    /// Updates an existing record in the database
    async fn update<T>(&self, entity: T) -> Result<bool, Error>
    where
        T: SqlCommand + UpdateParams + Send + Sync + 'static;

    /// Deletes a record from the database
    async fn delete<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync + 'static;

    /// Retrieves a single record from the database
    async fn fetch<P, R>(&self, params: P) -> Result<R, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync + 'static,
        R: FromRow + Send + Sync + 'static;

    /// Retrieves multiple records from the database
    async fn fetch_all<P, R>(&self, params: P) -> Result<Vec<R>, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync + 'static,
        R: FromRow + Send + Sync + 'static;
}

#[async_trait::async_trait]
impl PoolExtensions for Pool {
    async fn insert<T, P: for<'a> FromSql<'a> + Send + Sync>(&self, entity: T) -> Result<P, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync + 'static,
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;

        let sql = T::query();

        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled =
            *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

        if is_trace_enabled {
            println!("[PARSQL-DEADPOOL-POSTGRES] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let row = client.query_one(&sql, &params).await?;
        row.try_get::<_, P>(0)
    }

    async fn update<T>(&self, entity: T) -> Result<bool, Error>
    where
        T: SqlCommand + UpdateParams + Send + Sync + 'static,
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;

        let sql = T::query();

        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled =
            *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

        if is_trace_enabled {
            println!("[PARSQL-DEADPOOL-POSTGRES] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let result = client.execute(&sql, &params).await?;
        Ok(result > 0)
    }

    async fn delete<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync + 'static,
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;

        let sql = T::query();

        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled =
            *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

        if is_trace_enabled {
            println!("[PARSQL-DEADPOOL-POSTGRES] Execute SQL: {}", sql);
        }

        let params = entity.params();
        client.execute(&sql, &params).await
    }

    async fn fetch<P, R>(&self, params: P) -> Result<R, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync + 'static,
        R: FromRow + Send + Sync + 'static,
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;

        let sql = P::query();

        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled =
            *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

        if is_trace_enabled {
            println!("[PARSQL-DEADPOOL-POSTGRES] Execute SQL: {}", sql);
        }

        let query_params = params.params();
        let row = client.query_one(&sql, &query_params).await?;
        R::from_row(&row)
    }

    async fn fetch_all<P, R>(&self, params: P) -> Result<Vec<R>, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync + 'static,
        R: FromRow + Send + Sync + 'static,
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;

        let sql = P::query();

        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled =
            *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

        if is_trace_enabled {
            println!("[PARSQL-DEADPOOL-POSTGRES] Execute SQL: {}", sql);
        }

        let query_params = params.params();
        let rows = client.query(&sql, &query_params).await?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(R::from_row(&row)?);
        }

        Ok(results)
    }
}
