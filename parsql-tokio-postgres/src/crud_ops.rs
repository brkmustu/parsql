use crate::traits::{CrudOps, FromRow, SqlCommand, SqlParams, SqlQuery, UpdateParams};
use postgres::types::{FromSql, ToSql};
use std::sync::OnceLock;
use tokio_postgres::{Client, Error, Row, Transaction};

#[async_trait::async_trait]
impl CrudOps for Client {
    async fn insert<T, P: for<'a> FromSql<'a> + Send + Sync>(&self, entity: T) -> Result<P, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync + 'static,
    {
        insert(self, entity).await
    }

    async fn update<T>(&self, entity: T) -> Result<bool, Error>
    where
        T: SqlCommand + UpdateParams + Send + Sync + 'static,
    {
        update(self, entity).await
    }

    async fn delete<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync + 'static,
    {
        delete(self, entity).await
    }

    async fn fetch<P, R>(&self, params: P) -> Result<R, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync + 'static,
        R: FromRow + Send + Sync + 'static,
    {
        fetch(self, params).await
    }

    async fn fetch_all<P, R>(&self, params: P) -> Result<Vec<R>, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync + 'static,
        R: FromRow + Send + Sync + 'static,
    {
        fetch_all(self, params).await
    }

    async fn select<T, F, R>(&self, entity: T, to_model: F) -> Result<R, Error>
    where
        T: SqlQuery<T> + SqlParams + Send + Sync + 'static,
        F: Fn(&Row) -> Result<R, Error> + Send + Sync + 'static,
        R: Send + 'static,
    {
        select(self, entity, to_model).await
    }

    async fn select_all<T, F, R>(&self, entity: T, to_model: F) -> Result<Vec<R>, Error>
    where
        T: SqlQuery<T> + SqlParams + Send + Sync + 'static,
        F: Fn(&Row) -> R + Send + Sync + 'static,
        R: Send + 'static,
    {
        select_all(self, entity, to_model).await
    }
}

/// # insert
///
/// Inserts a new record into the database.
///
/// ## Parameters
/// - `client`: Database connection object
/// - `entity`: Data object to be inserted (must implement SqlQuery and SqlParams traits)
///
/// ## Return Value
/// - `Result<u64, Error>`: On success, returns the number of inserted records; on failure, returns Error
pub async fn insert<T, P: for<'a> FromSql<'a> + Send + Sync>(
    client: &Client,
    entity: T,
) -> Result<P, Error>
where
    T: SqlCommand + SqlParams + Send + Sync + 'static,
{
    let sql = T::query();

    static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
    let is_trace_enabled =
        *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

    if is_trace_enabled {
        println!("[PARSQL-TOKIO-POSTGRES] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let row = client.query_one(&sql, &params).await?;
    row.try_get::<_, P>(0)
}

/// # update
///
/// Updates an existing record in the database.
///
/// ## Parameters
/// - `client`: Database connection object
/// - `entity`: Data object containing the update information (must implement SqlQuery and UpdateParams traits)
///
/// ## Return Value
/// - `Result<bool, Error>`: On success, returns true; on failure, returns Error
pub async fn update<T>(client: &Client, entity: T) -> Result<bool, Error>
where
    T: SqlCommand + UpdateParams + Send + Sync + 'static,
{
    let sql = T::query();

    static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
    let is_trace_enabled =
        *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

    if is_trace_enabled {
        println!("[PARSQL-TOKIO-POSTGRES] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let result = client.execute(&sql, &params).await?;
    Ok(result > 0)
}

/// # delete
///
/// Deletes a record from the database.
///
/// ## Parameters
/// - `client`: Database connection object
/// - `entity`: Data object containing delete conditions (must implement SqlQuery and SqlParams traits)
///
/// ## Return Value
/// - `Result<u64, Error>`: On success, returns the number of deleted records; on failure, returns Error
pub async fn delete<T>(client: &Client, entity: T) -> Result<u64, Error>
where
    T: SqlCommand + SqlParams + Send + Sync + 'static,
{
    let sql = T::query();

    static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
    let is_trace_enabled =
        *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

    if is_trace_enabled {
        println!("[PARSQL-TOKIO-POSTGRES] Execute SQL: {}", sql);
    }

    let params = entity.params();
    client.execute(&sql, &params).await
}

/// # fetch
///
/// Retrieves a single record from the database and converts it to a struct.
///
/// ## Parameters
/// - `client`: Database connection object
/// - `params`: Data object containing query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
///
/// ## Return Value
/// - `Result<T, Error>`: On success, returns the retrieved record as a struct; on failure, returns Error
pub async fn fetch<P, R>(client: &Client, params: P) -> Result<R, Error>
where
    P: SqlQuery<R> + SqlParams + Send + Sync + 'static,
    R: FromRow + Send + Sync + 'static,
{
    let sql = P::query();

    static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
    let is_trace_enabled =
        *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

    if is_trace_enabled {
        println!("[PARSQL-TOKIO-POSTGRES] Execute SQL: {}", sql);
    }

    let query_params = params.params();
    let row = client.query_one(&sql, &query_params).await?;
    R::from_row(&row)
}

/// # fetch_all
///
/// Retrieves multiple records from the database.
///
/// ## Parameters
/// - `client`: Database connection object
/// - `params`: Query parameter object (must implement SqlQuery, FromRow, and SqlParams traits)
///
/// ## Return Value
/// - `Result<Vec<T>, Error>`: On success, returns the list of found records; on failure, returns Error
pub async fn fetch_all<P, R>(client: &Client, params: P) -> Result<Vec<R>, Error>
where
    P: SqlQuery<R> + SqlParams + Send + Sync + 'static,
    R: FromRow + Send + Sync + 'static,
{
    let sql = P::query();

    static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
    let is_trace_enabled =
        *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

    if is_trace_enabled {
        println!("[PARSQL-TOKIO-POSTGRES] Execute SQL: {}", sql);
    }

    let query_params = params.params();
    let rows = client.query(&sql, &query_params).await?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        results.push(R::from_row(&row)?);
    }

    Ok(results)
}

/// # select
///
/// Retrieves a single record from the database using a custom transformation function.
/// This is useful when you want to use a custom transformation function instead of the FromRow trait.
///
/// ## Parameters
/// - `client`: Database connection object
/// - `entity`: Query parameter object (must implement SqlQuery and SqlParams traits)
/// - `to_model`: Function to convert a Row object to the target object type
///
/// ## Return Value
/// - `Result<R, Error>`: On success, returns the transformed object; on failure, returns Error
pub async fn select<T, F, R>(client: &Client, entity: T, to_model: F) -> Result<R, Error>
where
    T: SqlQuery<T> + SqlParams + Send + Sync + 'static,
    F: Fn(&Row) -> Result<R, Error> + Send + Sync + 'static,
    R: Send + 'static,
{
    let sql = T::query();

    static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
    let is_trace_enabled =
        *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

    if is_trace_enabled {
        println!("[PARSQL-TOKIO-POSTGRES] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let row = client.query_one(&sql, &params).await?;
    to_model(&row)
}

/// # select_all
///
/// Retrieves multiple records from the database using a custom transformation function.
/// This is useful when you want to use a custom transformation function instead of the FromRow trait.
///
/// ## Parameters
/// - `client`: Database connection object
/// - `entity`: Query parameter object (must implement SqlQuery and SqlParams traits)
/// - `to_model`: Function to convert a Row object to the target object type
///
/// ## Return Value
/// - `Result<Vec<R>, Error>`: On success, returns the list of transformed objects; on failure, returns Error
pub async fn select_all<T, F, R>(client: &Client, entity: T, to_model: F) -> Result<Vec<R>, Error>
where
    T: SqlQuery<T> + SqlParams + Send + Sync + 'static,
    F: Fn(&Row) -> R + Send + Sync + 'static,
    R: Send + 'static,
{
    let sql = T::query();

    static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
    let is_trace_enabled =
        *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

    if is_trace_enabled {
        println!("[PARSQL-TOKIO-POSTGRES] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let rows = client.query(&sql, &params).await?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        results.push(to_model(&row));
    }

    Ok(results)
}

/// # get
///
/// Retrieves a single record from the database and converts it to a struct.
///
/// # Deprecated
/// This function has been renamed to `fetch`. Please use `fetch` instead.
///
/// # Arguments
/// * `client` - Database connection client
/// * `params` - Query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
///
/// # Return Value
/// * `Result<T, Error>` - On success, returns the retrieved record; on failure, returns Error
#[deprecated(
    since = "0.2.0",
    note = "Renamed to `fetch`. Please use `fetch` function instead."
)]
pub async fn get<T>(client: &Client, params: T) -> Result<T, Error>
where
    T: SqlQuery<T> + FromRow + SqlParams + Send + Sync + 'static,
{
    fetch(client, params).await
}

/// # get_all
///
/// Retrieves multiple records from the database.
///
/// # Deprecated
/// This function has been renamed to `fetch_all`. Please use `fetch_all` instead.
///
/// # Arguments
/// * `client` - Database connection client
/// * `params` - Query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
///
/// # Return Value
/// * `Result<Vec<T>, Error>` - On success, returns the list of found records; on failure, returns Error
#[deprecated(
    since = "0.2.0",
    note = "Renamed to `fetch_all`. Please use `fetch_all` function instead."
)]
pub async fn get_all<T>(client: &Client, params: T) -> Result<Vec<T>, Error>
where
    T: SqlQuery<T> + FromRow + SqlParams + Send + Sync + 'static,
{
    fetch_all(client, params).await
}
