use crate::traits::{FromRow, SqlCommand, SqlParams, SqlQuery, TransactionOps, UpdateParams};
use deadpool_postgres::{GenericClient, Transaction};
use std::fmt::Debug;
use std::sync::OnceLock;
use tokio_postgres::Row;
use tokio_postgres::{types::FromSql, Error};

/// Transaction extension trait for additional query operations
#[async_trait::async_trait]
pub trait TransactionExtensions {
    /// Inserts a new record into the database within a transaction
    async fn insert<T, P: for<'a> FromSql<'a> + Send + Sync>(&self, entity: T) -> Result<P, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync + 'static;

    /// Updates an existing record in the database within a transaction
    async fn update<T>(&self, entity: T) -> Result<bool, Error>
    where
        T: SqlCommand + UpdateParams + Send + Sync + 'static;

    /// Deletes a record from the database within a transaction
    async fn delete<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync + 'static;

    /// Retrieves a single record from the database within a transaction
    async fn fetch<P, R>(&self, params: P) -> Result<R, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync + 'static,
        R: FromRow + Send + Sync + 'static;

    /// Retrieves multiple records from the database within a transaction
    async fn fetch_all<P, R>(&self, params: P) -> Result<Vec<R>, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync + 'static,
        R: FromRow + Send + Sync + 'static;
}

#[async_trait::async_trait]
impl<'a> TransactionOps for Transaction<'a> {
    async fn tx_insert<T, P>(&self, entity: T) -> Result<P, Error>
    where
        T: SqlCommand + SqlParams + Debug + Send + 'static,
        P: for<'b> tokio_postgres::types::FromSql<'b> + Send + Sync,
    {
        let sql = T::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let query_params = entity.params();
        let row = self.query_one(&sql, &query_params).await?;
        row.try_get::<_, P>(0)
    }

    async fn tx_update<T>(&self, entity: T) -> Result<bool, Error>
    where
        T: SqlCommand + UpdateParams + SqlParams + Debug + Send + 'static,
    {
        let sql = T::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let query_params = <T as UpdateParams>::params(&entity);
        let result = self.execute(&sql, &query_params).await?;
        Ok(result > 0)
    }

    async fn tx_delete<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + SqlParams + Debug + Send + 'static,
    {
        let sql = T::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let query_params = entity.params();
        self.execute(&sql, &query_params).await
    }

    async fn tx_fetch<P, R>(&self, params: &P) -> Result<R, Error>
    where
        P: SqlQuery<R> + SqlParams + Debug + Send + Sync + Clone + 'static,
        R: FromRow + Debug + Send + Sync + Clone + 'static,
    {
        let sql = P::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let query_params = params.params();
        let row = self.query_one(&sql, &query_params).await?;
        R::from_row(&row)
    }

    async fn tx_fetch_all<P, R>(&self, params: &P) -> Result<Vec<R>, Error>
    where
        P: SqlQuery<R> + SqlParams + Debug + Send + Sync + Clone + 'static,
        R: FromRow + Debug + Send + Sync + Clone + 'static,
    {
        let sql = P::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let query_params = params.params();
        let rows = self.query(&sql, &query_params).await?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(R::from_row(&row)?);
        }

        Ok(results)
    }

    async fn tx_select<T, F, R>(&self, entity: T, to_model: F) -> Result<R, Error>
    where
        T: SqlQuery<T> + SqlParams + Debug + Send + 'static,
        F: Fn(&Row) -> Result<R, Error> + Send + Sync + 'static,
        R: Send + 'static,
    {
        let sql = T::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let query_params = entity.params();
        let row = self.query_one(&sql, &query_params).await?;
        to_model(&row)
    }

    async fn tx_select_all<T, F, R>(&self, entity: T, to_model: F) -> Result<Vec<R>, Error>
    where
        T: SqlQuery<T> + SqlParams + Debug + Send + 'static,
        F: Fn(&Row) -> R + Send + Sync + 'static,
        R: Send + 'static,
    {
        let sql = T::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let query_params = entity.params();
        let rows = self.query(&sql, &query_params).await?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(to_model(&row));
        }

        Ok(results)
    }

    // Deprecated methods for backward compatibility
    async fn insert<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + SqlParams + Debug + Send + 'static,
    {
        let sql = T::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let query_params = entity.params();
        self.execute(&sql, &query_params).await
    }

    async fn update<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + UpdateParams + SqlParams + Debug + Send + 'static,
    {
        let sql = T::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let query_params = <T as UpdateParams>::params(&entity);
        self.execute(&sql, &query_params).await
    }

    async fn delete<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + SqlParams + Debug + Send + 'static,
    {
        let sql = T::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let query_params = entity.params();
        self.execute(&sql, &query_params).await
    }

    async fn get<T>(&self, params: &T) -> Result<T, Error>
    where
        T: SqlQuery<T> + FromRow + SqlParams + Debug + Send + Sync + Clone + 'static,
    {
        self.tx_fetch(params).await
    }

    async fn get_all<T>(&self, params: &T) -> Result<Vec<T>, Error>
    where
        T: SqlQuery<T> + FromRow + SqlParams + Debug + Send + Sync + Clone + 'static,
    {
        self.tx_fetch_all(params).await
    }

    async fn select<T, R, F>(&self, entity: T, to_model: F) -> Result<R, Error>
    where
        T: SqlQuery<T> + SqlParams + Debug + Send + 'static,
        F: Fn(&Row) -> Result<R, Error> + Send + Sync + 'static,
        R: Send + 'static,
    {
        self.tx_select(entity, to_model).await
    }

    async fn select_all<T, R, F>(&self, entity: T, to_model: F) -> Result<Vec<R>, Error>
    where
        T: SqlQuery<T> + SqlParams + Debug + Send + 'static,
        F: Fn(&Row) -> R + Send + Sync + 'static,
        R: Send + 'static,
    {
        self.tx_select_all(entity, to_model).await
    }
}

#[async_trait::async_trait]
impl TransactionExtensions for Transaction<'_> {
    async fn insert<T, P: for<'a> FromSql<'a> + Send + Sync>(&self, entity: T) -> Result<P, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync + 'static,
    {
        let sql = T::query();

        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled =
            *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

        if is_trace_enabled {
            println!("[PARSQL-DEADPOOL-POSTGRES-TX] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let row = self.query_one(&sql, &params).await?;
        row.try_get::<_, P>(0)
    }

    async fn update<T>(&self, entity: T) -> Result<bool, Error>
    where
        T: SqlCommand + UpdateParams + Send + Sync + 'static,
    {
        let sql = T::query();

        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled =
            *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

        if is_trace_enabled {
            println!("[PARSQL-DEADPOOL-POSTGRES-TX] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let result = self.execute(&sql, &params).await?;
        Ok(result > 0)
    }

    async fn delete<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync + 'static,
    {
        let sql = T::query();

        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled =
            *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

        if is_trace_enabled {
            println!("[PARSQL-DEADPOOL-POSTGRES-TX] Execute SQL: {}", sql);
        }

        let params = entity.params();
        self.execute(&sql, &params).await
    }

    async fn fetch<P, R>(&self, params: P) -> Result<R, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync + 'static,
        R: FromRow + Send + Sync + 'static,
    {
        let sql = P::query();

        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled =
            *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

        if is_trace_enabled {
            println!("[PARSQL-DEADPOOL-POSTGRES-TX] Execute SQL: {}", sql);
        }

        let query_params = params.params();
        let row = self.query_one(&sql, &query_params).await?;
        R::from_row(&row)
    }

    async fn fetch_all<P, R>(&self, params: P) -> Result<Vec<R>, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync + 'static,
        R: FromRow + Send + Sync + 'static,
    {
        let sql = P::query();

        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled =
            *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

        if is_trace_enabled {
            println!("[PARSQL-DEADPOOL-POSTGRES-TX] Execute SQL: {}", sql);
        }

        let query_params = params.params();
        let rows = self.query(&sql, &query_params).await?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(R::from_row(&row)?);
        }

        Ok(results)
    }
}
