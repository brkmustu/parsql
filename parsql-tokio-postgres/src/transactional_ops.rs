// use parsql_core::{Deleteable, Insertable, Queryable, Updateable};
use deadpool_postgres::Transaction;
use tokio_postgres::{types::ToSql, Row, Error};

use crate::SqlParams;

pub async fn tx_update<T: Updateable + SqlParams>(
    transaction: Transaction<'_>,
    entity: T,
) -> Result<(Transaction<'_>, u64), Error> {
    let table = T::table();
    let columns = T::updated_columns();
    let where_clause = T::where_clause();

    let update = columns
        .iter()
        .enumerate()
        .map(|(i, col)| format!("{} = ${}", col, i + 1))
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!(
        "UPDATE {} SET {} WHERE {}",
        table, update, where_clause
    );

    let params = entity.params();

    let result = transaction.execute(&sql, &params).await?;
    Ok((transaction, result))
}

pub async fn tx_insert<T: Insertable + SqlParams>(
    transaction: Transaction<'_>,
    entity: T,
) -> Result<(Transaction<'_>, u64), Error> {
    let table = T::table();
    let columns = T::columns().join(", ");
    let placeholders = (1..=T::columns().len())
        .map(|i| format!("${}", i))
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table, columns, placeholders
    );

    let params = entity.params();

    let result = transaction.execute(&sql, &params).await?;
    Ok((transaction, result))
}