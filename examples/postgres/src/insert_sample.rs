use parsql::{
    macros::{Insertable, SqlParams},
    postgres::{SqlParams, SqlQuery},
};
use postgres::types::ToSql;

#[derive(Insertable, SqlParams)]
#[table("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}

#[derive(Insertable, SqlParams)]
#[table("posts")]
pub struct InsertPost {
    pub user_id: i32,
    pub content: String,
    pub state: i16,
}

#[derive(Insertable, SqlParams)]
#[table("comments")]
pub struct InsertComment {
    pub post_id: i32,
    pub content: String,
    pub state: i16,
}