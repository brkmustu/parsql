use parsql::{core::Updateable, macros::{UpdateParams, Updateable}, postgres::UpdateParams};
use postgres::types::ToSql;

#[derive(Updateable, UpdateParams)]
#[table_name("users")]
#[update_clause("name, email")]
#[where_clause("id = $")]
pub struct UpdateUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}
