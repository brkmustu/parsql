use parsql::{core::Insertable, macros::{Insertable, SqlParams}, postgres::SqlParams};
use postgres::types::ToSql;

#[derive(Insertable, SqlParams)]
#[table_name("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}
