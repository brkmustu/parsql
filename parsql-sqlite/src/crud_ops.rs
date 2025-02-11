use rusqlite::{Error, Row, ToSql};
use parsql_core::{SqlQuery, SqlParams, UpdateParams, FromRow};


pub fn insert<T: SqlQuery + SqlParams>(
    conn: &rusqlite::Connection,
    entity: T,
) -> Result<usize, rusqlite::Error> {
    let sql = T::query();

    let _params: Vec<&dyn ToSql> = entity.params().iter().map(|p| *p as &dyn ToSql).collect();

    match conn.execute(&sql, _params.as_slice()) {
        Ok(_result) => Ok(_result),
        Err(_err) => panic!("Insert işlemi yürütme esnasında bir hata oluştu! {}", _err),
    }
}

pub fn update<T: SqlQuery + UpdateParams>(
    conn: &rusqlite::Connection,
    entity: T,
) -> Result<usize, Error> {
    let sql = T::query();

    let _params: Vec<&dyn ToSql> = entity.params().iter().map(|p| *p as &dyn ToSql).collect();

    match conn.execute(&sql, _params.as_slice()) {
        Ok(rows_affected) => Ok(rows_affected),
        Err(e) => Err(e),
    }
}

pub fn delete<T: SqlQuery + SqlParams>(
    conn: &rusqlite::Connection,
    entity: T,
) -> Result<usize, Error> {
    let sql = T::query();

    let _params: Vec<&dyn ToSql> = entity.params().iter().map(|p| *p as &dyn ToSql).collect();

    match conn.execute(&sql, _params.as_slice()) {
        Ok(rows_affected) => Ok(rows_affected),
        Err(e) => Err(e),
    }
}

pub fn get<T: SqlQuery + FromRow + SqlParams>(
    conn: &rusqlite::Connection,
    entity: T,
) -> Result<T, Error> {
    let sql = T::query();

    let _params: Vec<&dyn ToSql> = entity.params().iter().map(|p| *p as &dyn ToSql).collect();

    conn.query_row(&sql, _params.as_slice(), |row| T::from_row(row))
}

pub fn get_all<T: SqlQuery + FromRow + SqlParams>(
    conn: &rusqlite::Connection,
    entity: T,
) -> Result<Vec<T>, Error> {
    let sql = T::query();
    let _params: Vec<&dyn ToSql> = entity.params().iter().map(|p| *p as &dyn ToSql).collect();
    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(_params.as_slice(), |row| T::from_row(row))?;
    let results = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(results)
}

pub fn select<T: SqlQuery + SqlParams, F>(
    conn: &mut rusqlite::Connection,
    entity: T,
    to_model: F,
) -> Result<T, Error>
where
    F: Fn(&Row) -> Result<T, Error>,
{

    let sql = T::query();

    let params: Vec<&dyn ToSql> = entity.params().iter().map(|p| *p as &dyn ToSql).collect();

    match conn.query_row(&sql, params.as_slice(), |row| to_model(row)) {
        Ok(row) => Ok(row),
        Err(e) => Err(e),
    }
}

pub fn select_all<T: SqlQuery + SqlParams, F>(
    conn: &mut rusqlite::Connection,
    entity: T,
    to_model: F,
) -> Result<Vec<T>, Error>
where
    F: Fn(&Row) -> Result<T, Error>,
{
    let sql = T::query();

    let params: Vec<&dyn ToSql> = entity.params().iter().map(|p| *p as &dyn ToSql).collect();

    let mut stmt = conn.prepare(&sql).unwrap();

    stmt.query_map(params.as_slice(), |row| to_model(row))
        .map(|iter| iter.collect::<Result<Vec<T>, _>>())
        .map_err(|err| println!("{:?}", err))
        .unwrap()
}
