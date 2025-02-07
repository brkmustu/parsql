use parsql::{
    macros::{Insertable, SqlParams},
    tokio_postgres::{SqlParams, SqlQuery},
};
use tokio_postgres::types::ToSql;

#[derive(Insertable, SqlParams)]
#[table_name("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}
