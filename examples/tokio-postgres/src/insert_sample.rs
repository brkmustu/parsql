use parsql::{macros::Insertable, Insertable};
use parsql_postgres::SqlParams;

#[derive(Insertable)]
#[table_name("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}