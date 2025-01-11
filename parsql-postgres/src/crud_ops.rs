use parsql_core::{Deleteable, Insertable, Queryable, Updateable};
use postgres::{types::ToSql, Client, Error, Row};

pub trait SqlParams {
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

pub trait UpdateParams {
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

pub trait FromRow {
    fn from_row(row: &Row) -> Self;
}

pub fn insert<T: Insertable + SqlParams>(client: &mut Client, entity: T) -> Result<u64, Error> {
    let table = T::table_name();
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

    client.execute(&sql, &params)
}

pub fn update<T: Updateable + UpdateParams>(
    client: &mut postgres::Client,
    entity: T,
) -> Result<u64, Error> {
    let table_name = T::table_name();
    let update_clause = T::update_clause();
    let where_clause = T::where_clause();

    // Sütunları "name = $1, age = $2" formatında birleştir
    let update_clause = update_clause
        .iter()
        .enumerate()
        .map(|(i, col)| format!("{} = ${}", col, i + 1))
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!(
        "UPDATE {} SET {} WHERE {}",
        table_name, update_clause, where_clause
    );

    let params = entity.params();

    match client.execute(&sql, &params) {
        Ok(rows_affected) => Ok(rows_affected),
        Err(e) => Err(e),
    }
}

pub fn delete<T: Deleteable + SqlParams>(
    client: &mut postgres::Client,
    entity: T,
) -> Result<u64, Error> {
    let table_name = T::table_name();
    let where_clause = T::where_clause();

    let sql = format!("DELETE FROM {} WHERE {}", table_name, where_clause);

    let params = entity.params();

    match client.execute(&sql, &params) {
        Ok(rows_affected) => Ok(rows_affected),
        Err(e) => Err(e),
    }
}

pub fn get<T: Queryable + FromRow + SqlParams>(
    client: &mut postgres::Client,
    entity: T,
) -> Result<T, Error>
{
    let table_name = T::table_name();
    let select_clause = T::select_clause().join(", ");
    let where_clause = T::where_clause();

    let sql = format!(
        "SELECT {} FROM {} WHERE {}",
        select_clause, table_name, where_clause
    );

    let params = entity.params();

    let query = client.prepare(&sql).unwrap();

    match client.query_one(&query, &params) {
        Ok(row) => Ok(T::from_row(&row)),
        Err(e) => Err(e),
    }
}

pub fn select<T: Queryable + SqlParams, F>(
    client: &mut postgres::Client,
    entity: T,
    to_model: F,
) -> Result<T, Error>
where
    F: Fn(&Row) -> Result<T, Error>,
{
    let table_name = T::table_name();
    let select_clause = T::select_clause().join(", ");
    let where_clause = T::where_clause();

    let sql = format!(
        "SELECT {} FROM {} WHERE {}",
        select_clause, table_name, where_clause
    );

    let params = entity.params();

    let query = client.prepare(&sql).unwrap();

    match client.query_one(&query, &params) {
        Ok(row) => Ok(to_model(&row)?),
        Err(e) => Err(e),
    }
}
