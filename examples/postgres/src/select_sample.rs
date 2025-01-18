use parsql::{
    core::Queryable,
    macros::{Queryable, SqlParams},
    postgres::SqlParams,
};
use postgres::types::ToSql;

#[derive(Queryable, SqlParams, Debug)]
#[table_name("users")]
#[where_clause("id = $")]
pub struct SelectUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}
