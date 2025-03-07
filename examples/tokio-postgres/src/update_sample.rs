use parsql::{
    macros::{UpdateParams, Updateable},
    tokio_postgres::{UpdateParams, SqlQuery},
};
use tokio_postgres::types::ToSql;

/// # UpdateUser
/// 
/// Data model used for updating an existing user record.
/// 
/// ## Attributes
/// - `#[derive(Updateable, UpdateParams)]`: Makes this type support update operations and update parameter generation.
/// - `#[table("users")]`: Specifies that this model will be used with the 'users' table.
/// - `#[update("name, email")]`: Specifies that only the name and email fields will be updated.
/// - `#[where_clause("id = $")]`: Specifies that the query will run with the 'WHERE id = ?' condition.
/// 
/// ## Fields
/// - `id`: Unique identifier of the user to update (used for the condition)
/// - `name`: User's name to update
/// - `email`: User's email address to update
/// - `state`: User's status (this field is not updated since it's not specified in the update attribute)
/// 
/// ## Usage
/// ```rust
/// // Prepare user information to update
/// let update_user = UpdateUser {
///     id: 1,
///     name: String::from("John"),
///     email: String::from("john@example.com"),
///     state: 2,
/// };
/// 
/// // Update in database
/// let update_result = update(&client, update_user).await;
/// ```
#[derive(Updateable, UpdateParams)]
#[table("users")]
#[update("name, email")]
#[where_clause("id = $")]
pub struct UpdateUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub state: i16,
}
