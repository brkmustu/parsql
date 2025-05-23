use chrono::{DateTime, Utc};
use parsql::deadpool_postgres::{
    traits::{SqlParams, SqlQuery, UpdateParams, FromRow},
    macros::{Insertable, Updateable, Queryable, Deletable, FromRow, SqlParams, UpdateParams},
};
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Row, Error};
use uuid::Uuid;

// Kullanıcı ekleme modeli
#[derive(Debug, Clone, Serialize, Deserialize, Insertable, SqlParams)]
#[table("users")]
#[returning("id")]
pub struct UserInsert {
    pub name: String,
    pub email: String,
    pub state: i16,
}

// Kullanıcı güncelleme modeli
#[derive(Debug, Clone, Serialize, Deserialize, Updateable, UpdateParams)]
#[table("users")]
#[update("name, email")]
#[where_clause("id = $")]
pub struct UserUpdate {
    pub id: i64,
    pub name: String,
    pub email: String,
}

// Kullanıcı silme modeli
#[derive(Debug, Clone, Serialize, Deserialize, Deletable, SqlParams)]
#[table("users")]
#[where_clause("id = $")]
pub struct UserDelete {
    pub id: i64,
}

// ID'ye göre kullanıcı getirme modeli
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, FromRow, SqlParams)]
#[table("users")]
#[select("id, name, email, state")]
#[where_clause("id = $")]
pub struct UserById {
    pub id: i64,
    pub name: String,
    pub email: String, 
    pub state: i16,
}

// State'e göre kullanıcıları getirme modeli
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, FromRow, SqlParams)]
#[table("users")]
#[select("id, name, email, state")]
#[where_clause("state = $")]
#[order_by("name ASC")]
pub struct UsersByState {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}

// Özel sorgu için model
#[derive(Queryable, SqlParams)]
#[table("users")]
#[select("id, name, email, CASE WHEN state = 1 THEN 'Aktif' ELSE 'Pasif' END as status")]
#[where_clause("state = $")]
pub struct UserStatusQuery {
    pub state: i16,
}

// Blog ekleme modeli
#[derive(Debug, Clone, Serialize, Deserialize, Insertable, SqlParams)]
#[table("blogs")]
#[returning("id")]
pub struct InsertBlog {
    pub title: String,
    pub content: Option<String>,
}

// Kullanıcı ekleme modeli için yardımcı metotlar
impl UserInsert {
    pub fn new(name: &str, email: &str, state: i16) -> Self {
        Self {
            name: name.to_string(),
            email: email.to_string(),
            state,
        }
    }
}

// Kullanıcı güncelleme modeli için yardımcı metotlar
impl UserUpdate {
    pub fn new(id: i64, name: &str, email: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            email: email.to_string(),
        }
    }
}

// Kullanıcı silme modeli için yardımcı metotlar
impl UserDelete {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

// ID'ye göre kullanıcı getirme modeli için yardımcı metotlar
impl UserById {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            name: String::new(),
            email: String::new(),
            state: 0,
        }
    }
}

// State'e göre kullanıcıları getirme modeli için yardımcı metotlar
impl UsersByState {
    pub fn new(state: i16) -> Self {
        Self {
            id: 0,
            name: String::new(),
            email: String::new(),
            state,
        }
    }
}

// Özel sorgu için yardımcı metotlar
impl UserStatusQuery {
    pub fn new(state: i16) -> Self {
        Self { state }
    }
}

// Blog ekleme modeli için yardımcı metotlar
impl InsertBlog {
    pub fn new(title: &str, content: Option<&str>) -> Self {
        Self {
            title: title.to_string(),
            content: content.map(|s| s.to_string()),
        }
    }
} 