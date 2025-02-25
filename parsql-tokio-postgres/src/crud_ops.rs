use tokio_postgres::{Error, Row};
use parsql_core::{SqlQuery, SqlParams, UpdateParams, FromRow};

pub async fn insert<T: SqlQuery + SqlParams>(
    client: &tokio_postgres::Client,
    entity: T,
) -> Result<u64, Error> {
    let sql = T::query();

    let params = entity.params();

    client.execute(&sql, &params).await
}

pub async fn update<T: SqlQuery + UpdateParams>(
    client: &tokio_postgres::Client,
    entity: T,
) -> Result<bool, Error> {
    let sql = T::query();

    let params = entity.params();

    match client.execute(&sql, &params).await {
        Ok(_) => Ok(true),
        Err(e) => Err(e),
    }
}

pub async fn delete<T: SqlQuery + SqlParams>(
    client: &tokio_postgres::Client,
    entity: T,
) -> Result<u64, Error> {
    let sql = T::query();

    println!("sql: {}", sql);

    let params = entity.params();

    match client.execute(&sql, &params).await {
        Ok(rows_affected) => Ok(rows_affected),
        Err(e) => Err(e),
    }
}

pub async fn get<T: SqlQuery + FromRow + SqlParams>(
    client: &tokio_postgres::Client,
    params: &T,
) -> Result<T, Error> {
    let sql = T::query();
    let params = params.params();
    match client.query_one(&sql, &params).await {
        Ok(_row) => T::from_row(&_row),
        Err(e) => Err(e),
    }
}

pub async fn get_all<T: SqlQuery + FromRow + SqlParams>(
    client: &tokio_postgres::Client,
    params: &T,
) -> Result<Vec<T>, Error> {
    let sql = T::query();
    println!("sql: {}", sql);
    let params = params.params();
    let rows = client.query(&sql, &params).await?;
    
    rows.iter()
        .map(|row| T::from_row(row))
        .collect::<Result<Vec<_>, _>>()
}

pub async fn select<T: SqlQuery + SqlParams, F>(
    client: &tokio_postgres::Client,
    entity: T,
    to_model: F,
) -> Result<T, Error>
where
    F: Fn(&Row) -> Result<T, Error>,
{
    let sql = T::query();

    let params = entity.params();

    match client.query_one(&sql, &params).await {
        Ok(_row) => to_model(&_row),
        Err(e) => Err(e),
    }
}

pub async fn select_all<T: SqlQuery + SqlParams, F>(
    client: &tokio_postgres::Client,
    entity: T,
    to_model: F,
) -> Result<Vec<T>, Error>
where
    F: Fn(&Row) -> T,
{
    let sql = T::query();

    let params = entity.params();

    let rows = client.query(&sql, &params).await?;

    let all_datas: Vec<T> = rows.iter().map(|row| to_model(row)).collect();

    Ok(all_datas)
}
