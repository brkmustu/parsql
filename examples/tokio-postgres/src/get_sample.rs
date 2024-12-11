use parsql::{macros::Queryable, Queryable};
use parsql_postgres::SqlParams;
use tokio_postgres::types::ToSql;

#[derive(Queryable, Debug)]
#[table_name("users")]
#[where_clause("id = $")]
pub struct GetUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}

impl GetUser {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            name: Default::default(),
            email: Default::default(),
            state: Default::default(),
        }
    }
}

#[derive(Queryable, Debug)]
#[table_name("users")]
#[where_clause("email = $")]
pub struct GetAllUsers {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}
