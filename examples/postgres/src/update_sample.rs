use parsql::{core::Updateable, macros::Updateable, postgres::SqlParams};
use postgres::types::ToSql;

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
