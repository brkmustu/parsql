use parsql::{macros::Deleteable, Deleteable};
use parsql_postgres::SqlParams;


#[derive(Deleteable, Debug)]
#[table_name("users")]
#[where_clause("id = $")]
pub struct DeleteUser {
    pub id: i64,
}