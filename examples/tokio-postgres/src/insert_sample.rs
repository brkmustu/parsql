use parsql::{
    macros::{Insertable, SqlParams},
    tokio_postgres::{SqlParams, SqlQuery},
};
use tokio_postgres::types::ToSql;

/// # InsertUser
/// 
/// Data model used for inserting a user.
/// 
/// ## Attributes
/// - `#[derive(Insertable, SqlParams)]`: Makes this type support insertion operations and SQL parameter generation.
/// - `#[table("users")]`: Specifies that this model will be used with the 'users' table.
/// 
/// ## Fields
/// - `name`: User's name
/// - `email`: User's email address
/// - `state`: User's status (active/inactive etc.)
/// 
/// ## Usage
/// ```rust
/// // Create a new user
/// let insert_user = InsertUser {
///     name: "John".to_string(),
///     email: "john@example.com".to_string(),
///     state: 1_i16,
/// };
/// 
/// // Insert into database
/// let insert_result = insert(&client, insert_user).await;
/// ```
#[derive(Insertable, SqlParams)]
#[table("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}
