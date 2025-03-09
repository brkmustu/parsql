use parsql::tokio_postgres::{delete, SqlParams, SqlQuery};
use tokio_postgres::{Client, types::ToSql};

/// # DeleteUser
/// 
/// A struct representing a user to be deleted from the database.
/// 
/// ## Attributes
/// 
/// - `#[table("users")]`: Specifies the database table name.
/// - `#[where_clause("id = $")]`: Defines the condition for deletion.
/// - `#[derive(Debug)]`: Enables debug formatting.
/// 
/// ## Fields
/// 
/// - `id`: The unique identifier of the user to delete.
/// 
/// ## Example
/// 
/// ```rust
/// let delete_user = DeleteUser { id: 1 };
/// let result = delete(&client, delete_user).await;
/// ```
#[derive(Debug)]
pub struct DeleteUser {
    pub id: i32,
}

impl SqlQuery for DeleteUser {
    fn query() -> String {
        "DELETE FROM users WHERE id = $1".to_string()
    }
}

impl SqlParams for DeleteUser {
    fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![&self.id]
    }
}

/// Demonstrates how to delete a user from the database.
pub async fn delete_sample(client: &Client) {
    // Create a user to delete
    let delete_user = DeleteUser { id: 1 };

    // Delete the user
    match delete(client, delete_user).await {
        Ok(count) => println!("Deleted {} user(s)", count),
        Err(e) => eprintln!("Error deleting user: {}", e),
    }
}
