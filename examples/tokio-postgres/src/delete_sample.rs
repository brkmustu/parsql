use parsql::{core::Deleteable, macros::{Deleteable, SqlParams}, tokio_postgres::SqlParams};
use tokio_postgres::types::ToSql;

#[derive(Deleteable, Debug, SqlParams)]
#[table_name("users")]
#[where_clause("id = $")]
pub struct DeleteUser {
    pub id: i64,
}