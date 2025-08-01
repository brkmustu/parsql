// use parsql_core::{Deleteable, Insertable, Queryable, Updateable};
use deadpool_postgres::{Client, Transaction};
use tokio_postgres::Error;
// Makrolar sadece dokümantasyon için kullanılıyor, gerçek kodda SqlQuery kullanılmalı
// use parsql_macros::{Insertable, Updateable};

use crate::traits::{CrudOps, FromRow, SqlCommand, SqlParams, SqlQuery, UpdateParams};

/// # begin
///
/// Starts a new database transaction from a pool client.
///
/// ## Parameters
/// - `client`: Pool client to start the transaction from
///
/// ## Return Value
/// - `Result<Transaction<'_>, Error>`: On success, returns the new transaction
///
/// ## Example Usage
/// ```rust,no_run
/// use tokio_postgres::{NoTls, Error};
/// use deadpool_postgres::{Config, Runtime};
/// use parsql::deadpool_postgres::transactional::begin;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let mut cfg = Config::new();
///     cfg.host = Some("localhost".to_string());
///     cfg.dbname = Some("test".to_string());
///     
///     let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
///     let mut client = pool.get().await?;
///     
///     let tx = begin(&mut client).await?;
///     // Now you can use the transaction
///     
///     // Commit the transaction
///     tx.commit().await?;
///     
///     Ok(())
/// }
/// ```
pub async fn begin(client: &mut Client) -> Result<Transaction<'_>, Error> {
    let tx = client.transaction().await?;

    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-TOKIO-POSTGRES-TX] Begin Transaction");
    }

    Ok(tx)
}

/// # begin_from_pool
///
/// Starts a new database transaction directly from a connection pool.
///
/// ## Parameters
/// - `pool`: Connection pool to start the transaction from
///
/// ## Return Value
/// - `Result<(Client, Transaction<'_>), Error>`: On success, returns the client and transaction
///
/// ## Example Usage
/// ```rust,no_run
/// use tokio_postgres::{NoTls, Error};
/// use deadpool_postgres::{Config, Runtime};
/// use parsql::deadpool_postgres::transactional::begin_from_pool;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let mut cfg = Config::new();
///     cfg.host = Some("localhost".to_string());
///     cfg.dbname = Some("test".to_string());
///     
///     let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
///     
///     // Instead of using begin_from_pool, we'll get a client and start a transaction
///     let client = pool.get().await.unwrap();
///     let tx = client.transaction().await?;
///     
///     // Now you can use the transaction
///     
///     // Commit the transaction
///     tx.commit().await?;
///     // Client will be dropped and returned to the pool automatically
///     
///     Ok(())
/// }
/// ```
// Bu fonksiyon lifetime sorunları nedeniyle kaldırıldı
// Bunun yerine client'ı manuel olarak alıp, üzerinde transaction başlatmak daha uygun

/// # tx_update
///
/// Updates a record within a transaction.
///
/// ## Parameters
/// - `transaction`: Active transaction object
/// - `entity`: Data object containing the update information (must implement Updateable and SqlParams traits)
///
/// ## Return Value
/// - `Result<(Transaction<'_>, u64), Error>`: On success, returns the transaction and number of updated records
///
/// ## Example Usage
/// ```rust,no_run
/// use tokio_postgres::{NoTls, Error};
/// use deadpool_postgres::{Config, Runtime};
/// use parsql::deadpool_postgres::transactional::tx_update;
///
/// #[derive(Updateable, UpdateParams)]
/// #[table("users")]
/// #[update("name, email")]
/// #[where_clause("id = $")]
/// pub struct UpdateUser {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let mut cfg = Config::new();
///     cfg.host = Some("localhost".to_string());
///     cfg.dbname = Some("test".to_string());
///     
///     let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
///     let client = pool.get().await?;
///     
///     let mut tx = client.transaction().await?;
///     
///     let update_user = UpdateUser {
///         id: 1,
///         name: String::from("John"),
///         email: String::from("john@example.com"),
///     };
///     
///     let (tx, rows_affected) = tx_update(tx, update_user).await?;
///     tx.commit().await?;
///     
///     println!("Updated {} rows", rows_affected);
///     Ok(())
/// }
/// ```
pub async fn tx_update<T: SqlCommand + SqlParams>(
    transaction: Transaction<'_>,
    entity: T,
) -> Result<(Transaction<'_>, u64), Error> {
    let sql = T::query();

    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-TOKIO-POSTGRES-TX] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let result = transaction.execute(&sql, &params).await?;
    Ok((transaction, result))
}

/// # tx_insert
///
/// Inserts a record within a transaction.
///
/// ## Parameters
/// - `transaction`: Active transaction object
/// - `entity`: Data object to be inserted (must implement Insertable and SqlParams traits)
///
/// ## Return Value
/// - `Result<(Transaction<'_>, u64), Error>`: On success, returns the transaction and number of inserted records
///
/// ## Example Usage
/// ```rust,no_run
/// use tokio_postgres::{NoTls, Error};
/// use deadpool_postgres::{Config, Runtime};
/// use parsql::deadpool_postgres::transactional::tx_insert;
///
/// #[derive(Insertable, SqlParams)]
/// #[table("users")]
/// pub struct InsertUser {
///     pub name: String,
///     pub email: String,
///     pub state: i16,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let mut cfg = Config::new();
///     cfg.host = Some("localhost".to_string());
///     cfg.dbname = Some("test".to_string());
///     
///     let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
///     let client = pool.get().await?;
///     
///     let mut tx = client.transaction().await?;
///     
///     let insert_user = InsertUser {
///         name: "John".to_string(),
///         email: "john@example.com".to_string(),
///         state: 1,
///     };
///     
///     let (tx, rows_affected) = tx_insert(tx, insert_user).await?;
///     tx.commit().await?;
///     
///     println!("Inserted {} rows", rows_affected);
///     Ok(())
/// }
/// ```
pub async fn tx_insert<T: SqlCommand + SqlParams>(
    transaction: Transaction<'_>,
    entity: T,
) -> Result<(Transaction<'_>, u64), Error> {
    let sql = T::query();

    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-TOKIO-POSTGRES-TX] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let result = transaction.execute(&sql, &params).await?;
    Ok((transaction, result))
}

/// # tx_delete
///
/// Deletes a record within a transaction.
///
/// ## Parameters
/// - `transaction`: Active transaction object
/// - `entity`: Data object identifying the record to delete (must implement Deletable and SqlParams traits)
///
/// ## Return Value
/// - `Result<(Transaction<'_>, u64), Error>`: On success, returns the transaction and number of deleted records
pub async fn tx_delete<T: SqlCommand + SqlParams>(
    transaction: Transaction<'_>,
    entity: T,
) -> Result<(Transaction<'_>, u64), Error> {
    let sql = T::query();

    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-TOKIO-POSTGRES-TX] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let result = transaction.execute(&sql, &params).await?;
    Ok((transaction, result))
}

/// # tx_get
///
/// Retrieves a single record within a transaction.
///
/// ## Parameters
/// - `transaction`: Active transaction object
/// - `params`: Query parameters (must implement SqlQuery, FromRow and SqlParams traits)
///
/// ## Return Value
/// - `Result<(Transaction<'_>, T), Error>`: On success, returns the transaction and the retrieved record
pub async fn tx_get<'a, T>(
    transaction: Transaction<'a>,
    params: &T,
) -> Result<(Transaction<'a>, T), Error>
where
    T: SqlQuery<T> + FromRow + SqlParams,
{
    let sql = T::query();

    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-TOKIO-POSTGRES-TX] Execute SQL: {}", sql);
    }

    let query_params = params.params();
    let row = transaction.query_one(&sql, &query_params).await?;
    let result = T::from_row(&row)?;

    Ok((transaction, result))
}

/// # tx_get_all
///
/// Retrieves multiple records within a transaction.
///
/// ## Parameters
/// - `transaction`: Active transaction object
/// - `params`: Query parameters (must implement SqlQuery, FromRow and SqlParams traits)
///
/// ## Return Value
/// - `Result<(Transaction<'_>, Vec<T>), Error>`: On success, returns the transaction and the retrieved records
pub async fn tx_get_all<'a, T>(
    transaction: Transaction<'a>,
    params: &T,
) -> Result<(Transaction<'a>, Vec<T>), Error>
where
    T: SqlQuery<T> + FromRow + SqlParams,
{
    let sql = T::query();

    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
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

/// # tx_select
///
/// Retrieves a single record using a custom transformation function within a transaction.
///
/// ## Parameters
/// - `transaction`: Active transaction object
/// - `entity`: Query parameters (must implement SqlQuery and SqlParams traits)
/// - `to_model`: Function to transform the row into the desired type
///
/// ## Return Value
/// - `Result<(Transaction<'_>, R), Error>`: On success, returns the transaction and the transformed record
pub async fn tx_select<'a, T, R, F>(
    transaction: Transaction<'a>,
    entity: T,
    to_model: F,
) -> Result<(Transaction<'a>, R), Error>
where
    T: SqlQuery<T> + SqlParams,
    F: FnOnce(&tokio_postgres::Row) -> Result<R, Error>,
{
    let sql = T::query();

    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-TOKIO-POSTGRES-TX] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let row = transaction.query_one(&sql, &params).await?;
    let result = to_model(&row)?;

    Ok((transaction, result))
}

/// # tx_select_all
///
/// Retrieves multiple records using a custom transformation function within a transaction.
///
/// ## Parameters
/// - `transaction`: Active transaction object
/// - `entity`: Query parameters (must implement SqlQuery and SqlParams traits)
/// - `to_model`: Function to transform each row into the desired type
///
/// ## Return Value
/// - `Result<(Transaction<'_>, Vec<R>), Error>`: On success, returns the transaction and the transformed records
pub async fn tx_select_all<'a, T, R, F>(
    transaction: Transaction<'a>,
    entity: T,
    to_model: F,
) -> Result<(Transaction<'a>, Vec<R>), Error>
where
    T: SqlQuery<T> + SqlParams,
    F: Fn(&tokio_postgres::Row) -> R,
{
    let sql = T::query();

    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-TOKIO-POSTGRES-TX] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let rows = transaction.query(&sql, &params).await?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        results.push(to_model(&row));
    }

    Ok((transaction, results))
}
