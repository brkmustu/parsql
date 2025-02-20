use parsql::{
    macros::{Queryable, SqlParams, FromRow},
    postgres::{SqlParams, SqlQuery, FromRow},
};
use postgres::{types::ToSql, Row, Error};

#[derive(Queryable, SqlParams, Debug)]
#[table("users")]
#[where_clause("id = $")]
pub struct SelectUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}

#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[select("users.id, users.name, users.email, users.state as user_state, posts.id as post_id, posts.content, posts.state as post_state")]
#[join("INNER JOIN posts ON users.id = posts.user_id")]
#[where_clause("users.id = $ and posts.state = 0")]
pub struct SelectUserWithPosts {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub user_state: i16,
    pub post_id: i64,
    pub content: String,
    pub post_state: i16,
}

impl SelectUserWithPosts {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            name: String::default(),
            email: String::default(),
            user_state: 0,
            post_id: 0,
            content: String::default(),
            post_state: 0,
        }
    }
}