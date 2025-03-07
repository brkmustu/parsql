use parsql::{
    macros::{Deleteable, SqlParams},
    tokio_postgres::{SqlParams, SqlQuery},
};
use tokio_postgres::types::ToSql;

/// # DeleteUser
/// 
/// Data model used for deleting a user record.
/// 
/// ## Attributes
/// - `#[derive(Deleteable, Debug, SqlParams)]`: Makes this type support deletion operations, debugging, and SQL parameter generation.
/// - `#[table("users")]`: Specifies that this model will be used with the 'users' table.
/// - `#[where_clause("id = $")]`: Specifies that the query will run with the 'WHERE id = ?' condition.
/// 
/// ## Fields
/// - `id`: Unique identifier of the user to delete
/// 
/// ## Usage
/// ```rust
/// // Specify the user ID to delete
/// let delete_user = DeleteUser { id: 6 };
/// 
/// // Delete from database
/// let delete_result = delete(&client, delete_user).await;
/// ```
#[derive(Deleteable, Debug, SqlParams)]
#[table("users")]
#[where_clause("id = $")]
pub struct DeleteUser {
    pub id: i32,
}
