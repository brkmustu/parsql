use parsql::{
    macros::{UpdateParams, Updateable},
    tokio_postgres::{UpdateParams, SqlQuery},
};
use tokio_postgres::types::ToSql;

#[derive(Updateable, UpdateParams)]
#[table("users")]
#[update("name, email")]
#[where_clause("id = $")]
pub struct UpdateUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub state: i16,
}
