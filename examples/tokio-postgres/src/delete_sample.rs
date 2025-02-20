use parsql::{
    macros::{Deleteable, SqlParams},
    tokio_postgres::{SqlParams, SqlQuery},
};
use tokio_postgres::types::ToSql;

#[derive(Deleteable, Debug, SqlParams)]
#[table("users")]
#[where_clause("id = $")]
pub struct DeleteUser {
    pub id: i64,
}
