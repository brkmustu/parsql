use parsql::{
    core::Queryable,
    macros::{Queryable, FromRow, SqlParams},
    postgres::{FromRow, SqlParams},
};
use postgres::{types::ToSql, Row, Error};

#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table_name("users")]
#[where_clause("id = $")]
pub struct SelectUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}
