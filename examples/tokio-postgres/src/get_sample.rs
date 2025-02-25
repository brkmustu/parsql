use parsql::{
    macros::{FromRow, Queryable, SqlParams},
    tokio_postgres::{FromRow, SqlParams, SqlQuery},
};
use tokio_postgres::{types::ToSql, Row, Error};

#[derive(Queryable, SqlParams, FromRow, Debug)]
#[table("users")]
#[where_clause("id = $")]
pub struct GetUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub state: i16,
}

impl GetUser {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            name: String::default(),
            email: String::default(),
            state: 0,
        }
    }
}

#[derive(Queryable, FromRow, Debug)]
#[table("users")]
#[where_clause("email = $")]
pub struct GetAllUsers {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub state: i16,
}

#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[select("users.id, users.name, users.email, users.state as user_state, posts.id as post_id, posts.content, posts.state as post_state, comments.content as comment")]
#[join("INNER JOIN posts ON users.id = posts.user_id")]
#[join("LEFT JOIN comments ON posts.id = comments.post_id")]
#[where_clause("users.id = $")]
pub struct SelectUserWithPosts {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub user_state: i16,
    pub post_id: i32,
    pub content: String,
    pub post_state: i16,
    pub comment: Option<String>,
}

impl SelectUserWithPosts {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            name: String::default(),
            email: String::default(),
            user_state: 0,
            post_id: 0,
            content: String::default(),
            post_state: 0,
            comment: None,
        }
    }
}
