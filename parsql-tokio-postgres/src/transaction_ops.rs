use crate::traits::{CrudOps, FromRow, SqlCommand, SqlParams, SqlQuery, UpdateParams};
use postgres::types::FromSql;
use std::sync::OnceLock;
use tokio_postgres::{Client, Error, Row, Transaction};

/// Creates and begins a new transaction.
///
/// This function is a wrapper around the tokio-postgres `transaction()` method.
/// It allows starting a new database transaction for performing multiple operations atomically.
///
/// # Return Value
/// * `Result<Transaction<'_>, Error>` - On success, returns the new transaction; on failure, returns Error
///
/// # Example
/// ```rust,no_run
/// # use tokio_postgres::{NoTls, Error};
/// # use parsql::tokio_postgres::transactional;
/// #
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let (mut client, connection) = tokio_postgres::connect("", NoTls).await?;
/// # tokio::spawn(async move { connection.await; });
/// let transaction = transactional::begin(&mut client).await?;
///
/// // Transaction işlemlerini gerçekleştir
///
/// transaction.commit().await?;
/// # Ok(())
/// # }
/// ```
pub async fn begin(client: &mut Client) -> Result<Transaction<'_>, Error> {
    client.transaction().await
}

/// Inserts a record within a transaction.
///
/// This function executes an INSERT SQL query within the given transaction.
/// It returns the transaction object, allowing for method chaining.
///
/// # Arguments
/// * `transaction` - An active transaction
/// * `entity` - Data object to be inserted (must implement SqlQuery and SqlParams traits)
///
/// # Return Value
/// * `Result<(Transaction<'_>, u64), Error>` - On success, returns the transaction and the number of affected rows; on failure, returns Error
///
/// # Example
/// ```rust,no_run
/// # use tokio_postgres::{NoTls, Error};
/// # use parsql::tokio_postgres::transactional;
/// # use parsql::macros::{Insertable, SqlParams};
/// #
/// #[derive(Insertable, SqlParams)]
/// #[table("users")]
/// struct InsertUser {
///     name: String,
///     email: String,
/// }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let (client, connection) = tokio_postgres::connect("", NoTls).await?;
/// # tokio::spawn(async move { connection.await; });
/// let user = InsertUser {
///     name: "John".to_string(),
///     email: "john@example.com".to_string(),
/// };
///
/// let transaction = transactional::begin(&client).await?;
/// let (transaction, rows_affected) = transactional::tx_insert(transaction, user).await?;
/// transaction.commit().await?;
/// # Ok(())
/// # }
/// ```
pub async fn tx_insert<T>(
    transaction: Transaction<'_>,
    entity: T,
) -> Result<(Transaction<'_>, u64), Error>
where
    T: SqlQuery<T> + SqlParams + Send + Sync + 'static,
{
    let sql = T::query();

    static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
    let is_trace_enabled =
        *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

    if is_trace_enabled {
        println!("[PARSQL-TOKIO-POSTGRES-TX] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let result = transaction.execute(&sql, &params).await?;
    Ok((transaction, result))
}

/// Updates a record within a transaction.
///
/// # Arguments
/// * `transaction` - An active transaction
/// * `entity` - Data object containing the update information (must implement SqlQuery and UpdateParams traits)
///
/// # Return Value
/// * `Result<(Transaction<'_>, bool), Error>` - On success, returns the transaction and whether any record was updated
///
/// # Example
/// ```rust,no_run
/// # use tokio_postgres::{NoTls, Error};
/// # use parsql::tokio_postgres::transactional;
/// # use parsql::macros::{Updateable, UpdateParams};
/// #
/// #[derive(Updateable, UpdateParams)]
/// #[table("users")]
/// #[update("name, email")]
/// #[where_clause("id = $")]
/// struct UpdateUser {
///     id: i64,
///     name: String,
///     email: String,
/// }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let (client, connection) = tokio_postgres::connect("", NoTls).await?;
/// # tokio::spawn(async move { connection.await; });
/// let user = UpdateUser {
///     id: 1,
///     name: "John Smith".to_string(),
///     email: "john.smith@example.com".to_string(),
/// };
///
/// let transaction = transactional::begin(&client).await?;
/// let (transaction, updated) = transactional::tx_update(transaction, user).await?;
/// transaction.commit().await?;
/// # Ok(())
/// # }
/// ```
pub async fn tx_update<T>(
    transaction: Transaction<'_>,
    entity: T,
) -> Result<(Transaction<'_>, bool), Error>
where
    T: SqlQuery<T> + UpdateParams + Send + Sync + 'static,
{
    let sql = T::query();

    static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
    let is_trace_enabled =
        *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

    if is_trace_enabled {
        println!("[PARSQL-TOKIO-POSTGRES-TX] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let result = transaction.execute(&sql, &params).await?;
    Ok((transaction, result > 0))
}

/// Deletes a record within a transaction.
///
/// # Arguments
/// * `transaction` - An active transaction
/// * `entity` - Data object containing delete conditions (must implement SqlQuery and SqlParams traits)
///
/// # Return Value
/// * `Result<(Transaction<'_>, u64), Error>` - On success, returns the transaction and number of deleted records
///
/// # Example
/// ```rust,no_run
/// # use tokio_postgres::{NoTls, Error};
/// # use parsql::tokio_postgres::transactional;
/// # use parsql::macros::{Deletable, SqlParams};
/// #
/// #[derive(Deletable, SqlParams)]
/// #[table("users")]
/// #[where_clause("id = $")]
/// struct DeleteUser {
///     id: i64,
/// }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let (client, connection) = tokio_postgres::connect("", NoTls).await?;
/// # tokio::spawn(async move { connection.await; });
/// let user = DeleteUser { id: 1 };
///
/// let transaction = transactional::begin(&client).await?;
/// let (transaction, deleted) = transactional::tx_delete(transaction, user).await?;
/// transaction.commit().await?;
/// # Ok(())
/// # }
/// ```
pub async fn tx_delete<T>(
    transaction: Transaction<'_>,
    entity: T,
) -> Result<(Transaction<'_>, u64), Error>
where
    T: SqlQuery<T> + SqlParams + Send + Sync + 'static,
{
    let sql = T::query();

    static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
    let is_trace_enabled =
        *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

    if is_trace_enabled {
        println!("[PARSQL-TOKIO-POSTGRES-TX] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let result = transaction.execute(&sql, &params).await?;
    Ok((transaction, result))
}

/// Retrieves a single record within a transaction.
///
/// # Arguments
/// * `transaction` - An active transaction
/// * `params` - Data object containing query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
///
/// # Return Value
/// * `Result<(Transaction<'_>, T), Error>` - On success, returns the transaction and the record
///
/// # Example
/// ```rust,no_run
/// # use tokio_postgres::{NoTls, Error};
/// # use parsql::tokio_postgres::transactional;
/// # use parsql::macros::{Queryable, FromRow, SqlParams};
/// #
/// #[derive(Queryable, FromRow, SqlParams, Debug)]
/// #[table("users")]
/// #[where_clause("id = $")]
/// struct GetUser {
///     id: i64,
///     name: String,
///     email: String,
/// }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let (client, connection) = tokio_postgres::connect("", NoTls).await?;
/// # tokio::spawn(async move { connection.await; });
/// let query = GetUser {
///     id: 1,
///     name: Default::default(),
///     email: Default::default(),
/// };
///
/// let transaction = transactional::begin(&client).await?;
/// let (transaction, user) = transactional::tx_fetch(transaction, query).await?;
/// transaction.commit().await?;
/// # Ok(())
/// # }
/// ```
pub async fn tx_fetch<T>(
    transaction: Transaction<'_>,
    params: T,
) -> Result<(Transaction<'_>, T), Error>
where
    T: SqlQuery<T> + FromRow + SqlParams + Send + Sync + 'static,
{
    let sql = T::query();

    static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
    let is_trace_enabled =
        *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

    if is_trace_enabled {
        println!("[PARSQL-TOKIO-POSTGRES-TX] Execute SQL: {}", sql);
    }

    let query_params = params.params();
    let row = transaction.query_one(&sql, &query_params).await?;
    let result = T::from_row(&row)?;
    Ok((transaction, result))
}

/// Retrieves multiple records within a transaction.
///
/// # Arguments
/// * `transaction` - An active transaction
/// * `params` - Data object containing query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
///
/// # Return Value
/// * `Result<(Transaction<'_>, Vec<T>), Error>` - On success, returns the transaction and a vector of records
///
/// # Example
/// ```rust,no_run
/// # use tokio_postgres::{NoTls, Error};
/// # use parsql::tokio_postgres::transactional;
/// # use parsql::macros::{Queryable, FromRow, SqlParams};
/// #
/// #[derive(Queryable, FromRow, SqlParams, Debug)]
/// #[table("users")]
/// #[where_clause("state = $")]
/// struct GetActiveUsers {
///     id: i64,
///     name: String,
///     email: String,
///     state: i16,
/// }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let (client, connection) = tokio_postgres::connect("", NoTls).await?;
/// # tokio::spawn(async move { connection.await; });
/// let query = GetActiveUsers {
///     id: 0,
///     name: Default::default(),
///     email: Default::default(),
///     state: 1, // active users
/// };
///
/// let transaction = transactional::begin(&client).await?;
/// let (transaction, users) = transactional::tx_fetch_all(transaction, query).await?;
/// transaction.commit().await?;
/// # Ok(())
/// # }
/// ```
pub async fn tx_fetch_all<T>(
    transaction: Transaction<'_>,
    params: T,
) -> Result<(Transaction<'_>, Vec<T>), Error>
where
    T: SqlQuery<T> + FromRow + SqlParams + Send + Sync + 'static,
{
    let sql = T::query();

    static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
    let is_trace_enabled =
        *TRACE_ENABLED.get_or_init(|| std::env::var("PARSQL_TRACE").unwrap_or_default() == "1");

    if is_trace_enabled {
        println!("[PARSQL-TOKIO-POSTGRES-TX] Execute SQL: {}", sql);
    }

    let query_params = params.params();
    let rows = transaction.query(&sql, &query_params).await?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        results.push(T::from_row(&row)?);
    }

    Ok((transaction, results))
}

/// Retrieves a single record within a transaction.
///
/// # Deprecated
/// This function has been renamed to `tx_fetch`. Please use `tx_fetch` instead.
///
/// # Arguments
/// * `transaction` - An active transaction
/// * `params` - Data object containing query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
///
/// # Return Value
/// * `Result<(Transaction<'_>, T), Error>` - On success, returns the transaction and the record
#[deprecated(
    since = "0.2.0",
    note = "Renamed to `tx_fetch`. Please use `tx_fetch` function instead."
)]
pub async fn tx_get<T>(
    transaction: Transaction<'_>,
    params: T,
) -> Result<(Transaction<'_>, T), Error>
where
    T: SqlQuery<T> + FromRow + SqlParams + Send + Sync + 'static,
{
    tx_fetch(transaction, params).await
}

/// Retrieves multiple records within a transaction.
///
/// # Deprecated
/// This function has been renamed to `tx_fetch_all`. Please use `tx_fetch_all` instead.
///
/// # Arguments
/// * `transaction` - An active transaction
/// * `params` - Data object containing query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
///
/// # Return Value
/// * `Result<(Transaction<'_>, Vec<T>), Error>` - On success, returns the transaction and a vector of records
#[deprecated(
    since = "0.2.0",
    note = "Renamed to `tx_fetch_all`. Please use `tx_fetch_all` function instead."
)]
pub async fn tx_get_all<T>(
    transaction: Transaction<'_>,
    params: T,
) -> Result<(Transaction<'_>, Vec<T>), Error>
where
    T: SqlQuery<T> + FromRow + SqlParams + Send + Sync + 'static,
{
    tx_fetch_all(transaction, params).await
}

/// Implementation of the CrudOps trait for Transactions
///
/// This implementation allows using the `CrudOps` trait methods directly on
/// `Transaction<'a>` objects, similar to how they are used on `Client` objects.
/// This provides a consistent API for both regular client operations and transaction operations.
///
/// # Examples
///
/// ```rust,no_run
/// # use tokio_postgres::{NoTls, Error};
/// # use parsql::tokio_postgres::{CrudOps};
/// # use parsql::macros::{Insertable, SqlParams};
/// #
/// # #[derive(Insertable, SqlParams)]
/// # #[table("users")]
/// # struct InsertUser {
/// #     name: String,
/// #     email: String,
/// # }
/// #
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let (mut client, connection) = tokio_postgres::connect("", NoTls).await?;
/// # tokio::spawn(async move { connection.await; });
/// let transaction = client.transaction().await?;
///
/// // Using CrudOps trait method directly on transaction
/// let user = InsertUser {
///     name: "John".to_string(),
///     email: "john@example.com".to_string(),
/// };
///
/// let rows_affected = transaction.insert(user).await?;
/// println!("Rows affected: {}", rows_affected);
///
/// transaction.commit().await?;
/// # Ok(())
/// # }
/// ```
#[async_trait::async_trait]
impl<'a> CrudOps for Transaction<'a> {
    async fn insert<T, P: for<'b> FromSql<'b> + Send + Sync>(&self, entity: T) -> Result<P, Error>
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
/// Inserts a new record into the database within a transaction.
///
/// ## Parameters
/// - `transaction`: Transaction object
/// - `entity`: Data object to be inserted (must implement SqlCommand and SqlParams traits)
///
/// ## Return Value
/// - `Result<P, Error>`: On success, returns the generated primary key; on failure, returns Error
pub async fn insert<T, P: for<'a> FromSql<'a> + Send + Sync>(
    transaction: &Transaction<'_>,
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
    let row = transaction.query_one(&sql, &params).await?;
    row.try_get::<_, P>(0)
}

/// # update
///
/// Updates an existing record in the database within a transaction.
///
/// ## Parameters
/// - `transaction`: Transaction object
/// - `entity`: Data object containing update information (must implement SqlCommand and UpdateParams traits)
///
/// ## Return Value
/// - `Result<bool, Error>`: On success, returns true if updated; on failure, returns Error
pub async fn update<T>(transaction: &Transaction<'_>, entity: T) -> Result<bool, Error>
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
    let result = transaction.execute(&sql, &params).await?;
    Ok(result > 0)
}

/// # delete
///
/// Deletes a record from the database within a transaction.
///
/// ## Parameters
/// - `transaction`: Transaction object
/// - `entity`: Data object containing delete conditions (must implement SqlCommand and SqlParams traits)
///
/// ## Return Value
/// - `Result<u64, Error>`: On success, returns the number of deleted records; on failure, returns Error
pub async fn delete<T>(transaction: &Transaction<'_>, entity: T) -> Result<u64, Error>
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
    transaction.execute(&sql, &params).await
}

/// # fetch
///
/// Retrieves a single record from the database within a transaction.
///
/// ## Parameters
/// - `transaction`: Transaction object
/// - `params`: Query parameters (must implement SqlQuery and SqlParams traits)
///
/// ## Return Value
/// - `Result<R, Error>`: On success, returns the retrieved record; on failure, returns Error
pub async fn fetch<P, R>(transaction: &Transaction<'_>, params: P) -> Result<R, Error>
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
    let row = transaction.query_one(&sql, &query_params).await?;
    R::from_row(&row)
}

/// # fetch_all
///
/// Retrieves multiple records from the database within a transaction.
///
/// ## Parameters
/// - `transaction`: Transaction object
/// - `params`: Query parameters (must implement SqlQuery and SqlParams traits)
///
/// ## Return Value
/// - `Result<Vec<R>, Error>`: On success, returns a vector of records; on failure, returns Error
pub async fn fetch_all<P, R>(transaction: &Transaction<'_>, params: P) -> Result<Vec<R>, Error>
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
    let rows = transaction.query(&sql, &query_params).await?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        results.push(R::from_row(&row)?);
    }

    Ok(results)
}

/// # select
///
/// Retrieves a single record from the database within a transaction using a custom transformation function.
///
/// ## Parameters
/// - `transaction`: Transaction object
/// - `entity`: Query parameter object (must implement SqlQuery and SqlParams traits)
/// - `to_model`: Function to convert a Row object to the target object type
///
/// ## Return Value
/// - `Result<R, Error>`: On success, returns the transformed object; on failure, returns Error
pub async fn select<T, F, R>(
    transaction: &Transaction<'_>,
    entity: T,
    to_model: F,
) -> Result<R, Error>
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
    let row = transaction.query_one(&sql, &params).await?;
    to_model(&row)
}

/// # select_all
///
/// Retrieves multiple records from the database within a transaction using a custom transformation function.
///
/// ## Parameters
/// - `transaction`: Transaction object
/// - `entity`: Query parameter object (must implement SqlQuery and SqlParams traits)
/// - `to_model`: Function to convert a Row object to the target object type
///
/// ## Return Value
/// - `Result<Vec<R>, Error>`: On success, returns a vector of transformed objects; on failure, returns Error
pub async fn select_all<T, F, R>(
    transaction: &Transaction<'_>,
    entity: T,
    to_model: F,
) -> Result<Vec<R>, Error>
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
    let rows = transaction.query(&sql, &params).await?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        results.push(to_model(&row));
    }

    Ok(results)
}
