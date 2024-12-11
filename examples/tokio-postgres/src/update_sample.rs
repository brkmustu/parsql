use parsql::{macros::Updateable, Updateable};
use parsql_postgres::SqlParams;
use tokio_postgres::types::ToSql;

#[derive(Updateable)]
#[table_name("users")]
#[update_clause("name, email")]
#[where_clause("id = $")]
pub struct UpdateUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}