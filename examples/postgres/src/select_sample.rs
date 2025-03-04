use parsql::{
    macros::{Queryable, SqlParams, FromRow},
    postgres::{SqlParams, SqlQuery, FromRow},
};
use postgres::{types::ToSql, Row, Error};

#[derive(Queryable, SqlParams, Debug)]
#[table("users")]
#[where_clause("id = $")]
pub struct SelectUser {
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

#[derive(Queryable, SqlParams, FromRow, Debug)]
#[table("users")]
#[select("users.state, COUNT(*) as user_count")]
#[where_clause("state > $")]
#[group_by("users.state")]
#[having("COUNT(*) > 0")]
#[order_by("user_count DESC")]
pub struct UserStateStats {
    pub state: i16,
    pub user_count: i64,
}

impl UserStateStats {
    pub fn new(min_state: i16) -> Self {
        Self {
            state: min_state,
            user_count: 0,
        }
    }
}

#[derive(Queryable, SqlParams, FromRow, Debug)]
#[table("users")]
#[select("users.state, posts.state as post_state, COUNT(posts.id) as post_count")]
#[join("LEFT JOIN posts ON users.id = posts.user_id")]
#[where_clause("users.state > $")]
#[group_by("users.state, posts.state")]
#[having("COUNT(posts.id) > 0")]
#[order_by("post_count DESC")]
pub struct UserPostStats {
    pub state: i16,
    pub post_state: Option<i16>,
    pub post_count: i64,
}

impl UserPostStats {
    pub fn new(min_state: i16) -> Self {
        Self {
            state: min_state,
            post_state: None,
            post_count: 0,
        }
    }
}