use postgres::{Client, Error, Row};
use postgres::types::ToSql;
use parsql_core::{SqlQuery, SqlParams, UpdateParams, FromRow};

pub fn insert<T: SqlQuery + SqlParams>(client: &mut Client, entity: T) -> Result<u64, Error> {
    let sql = T::query();

    let params = entity.params();

    client.execute(&sql, &params)
}

pub fn update<T: SqlQuery + UpdateParams>(
    client: &mut postgres::Client,
    entity: T,
) -> Result<u64, Error> {
    let sql = T::query();

    let params = entity.params();

    match client.execute(&sql, &params) {
        Ok(rows_affected) => Ok(rows_affected),
        Err(e) => Err(e),
    }
}

pub fn delete<T: SqlQuery + SqlParams>(
    client: &mut postgres::Client,
    entity: T,
) -> Result<u64, Error> {
    let sql = T::query();

    let params = entity.params();

    match client.execute(&sql, &params) {
        Ok(rows_affected) => Ok(rows_affected),
        Err(e) => Err(e),
    }
}

pub fn get<T: SqlQuery + FromRow + SqlParams>(
    client: &mut Client,
    params: &T,
) -> Result<T, Error> {
    let query = T::query();
    let params = params.params();
    match client.query_one(&query, &params) {
        Ok(row) => T::from_row(&row),
        Err(e) => Err(e),
    }
}

pub fn get_all<T: SqlQuery + FromRow + SqlParams>(
    client: &mut Client,
    params: &T,
) -> Result<Vec<T>, Error> {
    let query = T::query();
    let params = params.params();
    let rows = client.query(&query, &params)?;
    
    rows.iter()
        .map(|row| T::from_row(row))
        .collect()
}

pub fn get_by_query<T: FromRow>(
    client: &mut Client,
    query: &str,
    params: &[&(dyn ToSql + Sync)],
) -> Result<Vec<T>, Error> {
    let rows = client.query(query, params)?;
    
    rows.iter()
        .map(|row| T::from_row(row))
        .collect()
}

pub fn select<T: SqlQuery + SqlParams, F>(
    client: &mut postgres::Client,
    entity: T,
    to_model: F,
) -> Result<T, Error>
where
    F: Fn(&Row) -> Result<T, Error>,
{
    let sql = T::query();

    let params = entity.params();

    let query = client.prepare(&sql).unwrap();

    match client.query_one(&query, &params) {
        Ok(row) => Ok(to_model(&row)?),
        Err(e) => Err(e),
    }
}

pub fn select_all<T: SqlQuery + SqlParams, F>(
    client: &mut postgres::Client,
    entity: T,
    to_model: F,
) -> Result<Vec<T>, Error>
where
    F: Fn(&Row) -> Result<T, Error>,
{
    let sql = T::query();
    let params = entity.params();
    let rows = client.query(&sql, &params)?;

    rows.iter()
        .map(|row| to_model(row))
        .collect()
}
