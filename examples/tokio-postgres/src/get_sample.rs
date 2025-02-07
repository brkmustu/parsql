use parsql::{
    macros::{FromRow, Queryable, SqlParams},
    tokio_postgres::{FromRow, SqlParams, SqlQuery},
};
use tokio_postgres::{types::ToSql, Row, Error};

#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table_name("users")]
#[where_clause("id = $")]
pub struct GetUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}

impl GetUser {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            name: Default::default(),
            email: Default::default(),
            state: Default::default(),
        }
    }
}

#[derive(Queryable, FromRow, Debug)]
#[table_name("users")]
#[where_clause("email = $")]
pub struct GetAllUsers {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}
