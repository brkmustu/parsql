use postgres::{types::ToSql, Client, Error, Row};
use crate::{SqlQuery, SqlParams, UpdateParams, FromRow};

/// # insert
/// 
/// Inserts a new record into the database.
/// 
/// ## Parameters
/// - `client`: Database connection client
/// - `entity`: Data object to be inserted (must implement SqlQuery and SqlParams traits)
/// 
/// ## Return Value
/// - `Result<u64, Error>`: On success, returns the number of inserted records; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust,no_run
/// #[derive(Insertable, SqlParams)]  // Required macros
/// #[table("table_name")]            // Table name to insert into
/// pub struct MyEntity {
///     pub field1: String,
///     pub field2: i32,
///     // ...
/// }
/// ```
/// 
/// - `Insertable`: Automatically generates SQL INSERT statements
/// - `SqlParams`: Automatically generates SQL parameters
/// - `#[table("table_name")]`: Specifies the table name for the insertion
/// 
/// ## Example Usage
/// ```rust,no_run
/// use postgres::{Client, NoTls, Error};
/// use parsql::postgres::insert;
/// 
/// #[derive(Insertable, SqlParams)]
/// #[table("users")]
/// pub struct InsertUser {
///     pub name: String,
///     pub email: String,
///     pub state: i16,
/// }
///
/// fn main() -> Result<(), Error> {
///     let mut client = Client::connect(
///         "host=localhost user=postgres dbname=test",
///         NoTls,
///     )?;
///
///     let insert_user = InsertUser {
///         name: "John".to_string(),
///         email: "john@example.com".to_string(),
///         state: 1_i16,
///     };
///
///     let insert_result = insert(&mut client, insert_user)?;
///     println!("Insert result: {:?}", insert_result);
///     Ok(())
/// }
/// ```
pub fn insert<T: SqlQuery + SqlParams>(client: &mut Client, entity: T) -> Result<u64, Error> {
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-POSTGRES] Execute SQL: {}", sql);
    }

    let params = entity.params();
    client.execute(&sql, &params)
}

/// # update
/// 
/// Updates an existing record in the database.
/// 
/// ## Parameters
/// - `client`: Database connection client
/// - `entity`: Data object containing the update information (must implement SqlQuery and UpdateParams traits)
/// 
/// ## Return Value
/// - `Result<u64, Error>`: On success, returns the number of updated records; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust,no_run
/// #[derive(Updateable, UpdateParams)]  // Required macros
/// #[table("table_name")]               // Table name to update
/// #[update("field1, field2")]          // Fields to update (optional)
/// #[where_clause("id = $")]            // Update condition
/// pub struct MyEntity {
///     pub id: i32,                     // Fields used in the condition
///     pub field1: String,              // Fields to be updated
///     pub field2: i32,                 // Fields to be updated
///     // ...
/// }
/// ```
/// 
/// - `Updateable`: Automatically generates SQL UPDATE statements
/// - `UpdateParams`: Automatically generates update parameters
/// - `#[table("table_name")]`: Specifies the table name for the update
/// - `#[update("field1, field2")]`: Specifies which fields should be updated (if omitted, all fields will be updated)
/// - `#[where_clause("id = $")]`: Specifies the update condition (`$` will be replaced with parameter value)
/// 
/// ## Example Usage
/// ```rust,no_run
/// use postgres::{Client, NoTls, Error};
/// use parsql::postgres::update;
/// 
/// #[derive(Updateable, UpdateParams)]
/// #[table("users")]
/// #[update("name, email")]
/// #[where_clause("id = $")]
/// pub struct UpdateUser {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
///     pub state: i16,  // This field won't be updated as it's not specified in the update attribute
/// }
///
/// fn main() -> Result<(), Error> {
///     let mut client = Client::connect(
///         "host=localhost user=postgres dbname=test",
///         NoTls,
///     )?;
///
///     let update_user = UpdateUser {
///         id: 1,
///         name: String::from("John"),
///         email: String::from("john@example.com"),
///         state: 2,
///     };
///
///     let update_result = update(&mut client, update_user)?;
///     println!("Update result: {:?}", update_result);
///     Ok(())
/// }
/// ```
pub fn update<T: SqlQuery + UpdateParams>(
    client: &mut postgres::Client,
    entity: T,
) -> Result<u64, Error> {
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-POSTGRES] Execute SQL: {}", sql);
    }

    let params = entity.params();
    match client.execute(&sql, &params) {
        Ok(rows_affected) => Ok(rows_affected),
        Err(e) => Err(e),
    }
}

/// # delete
/// 
/// Deletes a record from the database.
/// 
/// ## Parameters
/// - `client`: Database connection client
/// - `entity`: Data object containing the deletion information (must implement SqlQuery and SqlParams traits)
/// 
/// ## Return Value
/// - `Result<u64, Error>`: On success, returns the number of deleted records; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust,no_run
/// #[derive(Deletable, SqlParams)]   // Required macros
/// #[table("table_name")]             // Table name to delete from
/// #[where_clause("id = $")]          // Delete condition
/// pub struct MyEntity {
///     pub id: i32,                   // Fields used in the condition
///     // Other fields can be added, but typically only condition fields are necessary
/// }
/// ```
/// 
/// - `Deletable`: Automatically generates SQL DELETE statements
/// - `SqlParams`: Automatically generates SQL parameters
/// - `#[table("table_name")]`: Specifies the table name for the deletion
/// - `#[where_clause("id = $")]`: Specifies the delete condition (`$` will be replaced with parameter value)
/// 
/// ## Example Usage
/// ```rust,no_run
/// use postgres::{Client, NoTls, Error};
/// use parsql::postgres::delete;
/// 
/// #[derive(Deletable, SqlParams)]
/// #[table("users")]
/// #[where_clause("id = $")]
/// pub struct DeleteUser {
///     pub id: i32,
/// }
/// 
/// fn main() -> Result<(), Error> {
///     let mut client = Client::connect(
///         "host=localhost user=postgres dbname=test",
///         NoTls,
///     )?;
///
///     let delete_user = DeleteUser { id: 6 };
///     let delete_result = delete(&mut client, delete_user)?;
///     
///     println!("Delete result: {:?}", delete_result);
///     Ok(())
/// }
/// ```
pub fn delete<T: SqlQuery + SqlParams>(
    client: &mut postgres::Client,
    entity: T,
) -> Result<u64, Error> {
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-POSTGRES] Execute SQL: {}", sql);
    }

    let params = entity.params();
    match client.execute(&sql, &params) {
        Ok(rows_affected) => Ok(rows_affected),
        Err(e) => Err(e),
    }
}

/// # get
/// 
/// Retrieves a single record from the database.
/// 
/// ## Parameters
/// - `client`: Database connection client
/// - `params`: Query parameter object (must implement SqlQuery, FromRow, and SqlParams traits)
/// 
/// ## Return Value
/// - `Result<T, Error>`: On success, returns the found record; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust,no_run
/// #[derive(Queryable, SqlParams, FromRow, Debug)]  // Required macros
/// #[table("table_name")]                           // Table name to query
/// #[where_clause("id = $")]                        // Query condition
/// pub struct MyEntity {
///     pub id: i32,                                 // Field used in the query condition
///     pub field1: String,                          // Fields to be populated from the result set
///     pub field2: i32,
///     // ...
/// }
/// 
/// // A factory method is also useful
/// impl MyEntity {
///     pub fn new(id: i32) -> Self {
///         Self {
///             id,
///             field1: String::default(),
///             field2: 0,
///             // ...
///         }
///     }
/// }
/// ```
/// 
/// - `Queryable`: Automatically generates SQL SELECT statements
/// - `SqlParams`: Automatically generates SQL parameters
/// - `FromRow`: Enables conversion from database row to struct object
/// - `#[table("table_name")]`: Specifies the table name for the query
/// - `#[where_clause("id = $")]`: Specifies the query condition (`$` will be replaced with parameter value)
/// 
/// ## Example Usage
/// ```rust,no_run
/// use postgres::{Client, NoTls, Error};
/// use parsql::postgres::get;
/// 
/// #[derive(Queryable, SqlParams, FromRow, Debug)]
/// #[table("users")]
/// #[where_clause("id = $")]
/// pub struct GetUser {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
///     pub state: i16,
/// }
/// 
/// impl GetUser {
///     pub fn new(id: i32) -> Self {
///         Self {
///             id,
///             name: String::default(),
///             email: String::default(),
///             state: 0,
///         }
///     }
/// }
///
/// fn main() -> Result<(), Error> {
///     let mut client = Client::connect(
///         "host=localhost user=postgres dbname=test",
///         NoTls,
///     )?;
///
///     let get_user = GetUser::new(1);
///     let get_result = get(&mut client, &get_user)?;
///     
///     println!("Get result: {:?}", get_result);
///     Ok(())
/// }
/// ```
pub fn get<T: SqlQuery + FromRow + SqlParams>(
    client: &mut Client,
    params: &T,
) -> Result<T, Error> {
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-POSTGRES] Execute SQL: {}", sql);
    }
    
    let params = params.params();
    match client.query_one(&sql, &params) {
        Ok(_row) => T::from_row(&_row),
        Err(e) => Err(e),
    }
}

/// # get_all
/// 
/// Retrieves multiple records from the database.
/// 
/// ## Parameters
/// - `client`: Database connection client
/// - `params`: Query parameter object (must implement SqlQuery, FromRow, and SqlParams traits)
/// 
/// ## Return Value
/// - `Result<Vec<T>, Error>`: On success, returns the list of found records; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust,no_run
/// #[derive(Queryable, SqlParams, FromRow, Debug)]  // Required macros
/// #[table("table_name")]                           // Table name to query
/// #[select("field1, field2, COUNT(*) as count")]   // Custom SELECT statement (optional)
/// #[join("INNER JOIN other_table ON ...")]         // JOIN statements (optional)
/// #[where_clause("status > $")]                    // Query condition
/// #[group_by("field1, field2")]                    // GROUP BY statement (optional)
/// #[having("COUNT(*) > 0")]                        // HAVING statement (optional)
/// #[order_by("count DESC")]                        // ORDER BY statement (optional)
/// pub struct MyEntity {
///     pub status: i32,                             // Field used in the query condition
///     pub field1: String,                          // Fields to be populated from the result set
///     pub field2: i32,
///     pub count: i64,                              // Calculated value
///     // ...
/// }
/// 
/// impl MyEntity {
///     pub fn new(status: i32) -> Self {
///         Self {
///             status,
///             field1: String::default(),
///             field2: 0,
///             count: 0,
///             // ...
///         }
///     }
/// }
/// ```
/// 
/// - `Queryable`: Automatically generates SQL SELECT statements
/// - `SqlParams`: Automatically generates SQL parameters
/// - `FromRow`: Enables conversion from database row to struct object
/// - `#[table("table_name")]`: Specifies the table name for the query
/// - `#[select("...")]`: Creates a custom SELECT statement (if omitted, all fields will be selected)
/// - `#[join("...")]`: Specifies JOIN statements (can be used multiple times)
/// - `#[where_clause("...")]`: Specifies the query condition (`$` will be replaced with parameter value)
/// - `#[group_by("...")]`: Specifies the GROUP BY statement
/// - `#[having("...")]`: Specifies the HAVING statement
/// - `#[order_by("...")]`: Specifies the ORDER BY statement
/// 
/// ## Example Usage
/// ```rust,no_run
/// use postgres::{Client, NoTls, Error};
/// use parsql::postgres::get_all;
/// 
/// // Simple query example
/// #[derive(Queryable, SqlParams, FromRow, Debug)]
/// #[table("users")]
/// #[where_clause("email = $")]
/// pub struct GetAllUsers {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
///     pub state: i16,
/// }
///
/// // Complex JOIN example
/// #[derive(Queryable, SqlParams, FromRow, Debug)]
/// #[table("users")]
/// #[select("users.id, users.name, users.email, users.state as user_state, posts.id as post_id, posts.content, posts.state as post_state, comments.content as comment")]
/// #[join("INNER JOIN posts ON users.id = posts.user_id")]
/// #[join("LEFT JOIN comments ON posts.id = comments.post_id")]
/// #[where_clause("users.id = $")]
/// pub struct SelectUserWithPosts {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
///     pub user_state: i16,
///     pub post_id: i32,
///     pub content: String,
///     pub post_state: i16,
///     pub comment: Option<String>,
/// }
///
/// // GROUP BY and ORDER BY example
/// #[derive(Queryable, SqlParams, FromRow, Debug)]
/// #[table("users")]
/// #[select("users.state, COUNT(*) as user_count")]
/// #[where_clause("state > $")]
/// #[group_by("users.state")]
/// #[order_by("user_count DESC")]
/// pub struct UserStateStats {
///     pub state: i16,
///     pub user_count: i64,
/// }
///
/// // HAVING filter example
/// #[derive(Queryable, SqlParams, FromRow, Debug)]
/// #[table("users")]
/// #[select("users.state, COUNT(*) as user_count")]
/// #[where_clause("state > $")]
/// #[group_by("users.state")]
/// #[having("COUNT(*) > 1")]
/// #[order_by("user_count DESC")]
/// pub struct UserStateStatsFiltered {
///     pub state: i16,
///     pub user_count: i64,
/// }
///
/// fn main() -> Result<(), Error> {
///     let mut client = Client::connect(
///         "host=localhost user=postgres dbname=test",
///         NoTls,
///     )?;
///
///     // Example usage
///     let select_user_with_posts = SelectUserWithPosts::new(1);
///     let get_user_with_posts = get_all(&mut client, &select_user_with_posts)?;
///     
///     println!("Get user with posts: {:?}", get_user_with_posts);
///     
///     // Other examples
///     let user_state_stats = get_all(&mut client, &UserStateStats::new(0))?;
///     println!("User state stats: {:?}", user_state_stats);
///     
///     let user_state_stats_filtered = get_all(&mut client, &UserStateStatsFiltered::new(0))?;
///     println!("User state stats (filtered with HAVING): {:?}", user_state_stats_filtered);
///     Ok(())
/// }
/// ```
pub fn get_all<T: SqlQuery + FromRow + SqlParams>(
    client: &mut Client,
    params: &T,
) -> Result<Vec<T>, Error> {
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-POSTGRES] Execute SQL: {}", sql);
    }
    let params = params.params();
    let rows = client.query(&sql, &params)?;
    
    rows.iter()
        .map(|row| T::from_row(row))
        .collect::<Result<Vec<_>, _>>()
}

/// # get_by_query
/// 
/// Retrieves multiple records from the database using a custom SQL query.
/// 
/// ## Parameters
/// - `client`: Database connection client
/// - `query`: Custom SQL query string
/// - `params`: Array of query parameters
/// 
/// ## Return Value
/// - `Result<Vec<T>, Error>`: On success, returns the list of found records; on failure, returns Error
/// 
/// ## Example Usage
/// ```rust,no_run
/// use postgres::{Client, NoTls, Error};
/// use parsql::postgres::get_by_query;
/// 
/// #[derive(FromRow, Debug)]
/// pub struct UserStats {
///     pub state: i16,
///     pub user_count: i64,
/// }
///
/// fn main() -> Result<(), Error> {
///     let mut client = Client::connect(
///         "host=localhost user=postgres dbname=test",
///         NoTls,
///     )?;
///
///     let query = "SELECT state, COUNT(*) as user_count FROM users GROUP BY state HAVING COUNT(*) > $1";
///     let min_count = 5;
///     
///     let stats = get_by_query::<UserStats>(&mut client, query, &[&min_count])?;
///     println!("User stats: {:?}", stats);
///     Ok(())
/// }
/// ```
pub fn get_by_query<T: FromRow>(
    client: &mut Client,
    query: &str,
    params: &[&(dyn ToSql + Sync)],
) -> Result<Vec<T>, Error> {
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-POSTGRES] Execute SQL: {}", query);
    }

    let rows = client.query(query, params)?;
    rows.iter()
        .map(|row| T::from_row(row))
        .collect::<Result<Vec<_>, _>>()
}

/// # select
/// 
/// Retrieves a single record from the database using a custom transformation function.
/// This is useful when you want to use a custom transformation function instead of the FromRow trait.
/// 
/// ## Parameters
/// - `client`: Database connection client
/// - `entity`: Query parameter object (must implement SqlQuery and SqlParams traits)
/// - `to_model`: Function to convert a Row object to the target object type
/// 
/// ## Return Value
/// - `Result<T, Error>`: On success, returns the transformed object; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust,no_run
/// #[derive(Queryable, SqlParams)]          // Required macros (FromRow is not needed)
/// #[table("table_name")]                   // Table name to query
/// #[where_clause("id = $")]                // Query condition
/// pub struct MyQueryEntity {
///     pub id: i32,                         // Field used in the query condition
///     // Other fields can be added if necessary for the query condition
/// }
/// 
/// // A separate struct can be used for the return value
/// pub struct MyResultEntity {
///     pub id: i32,
///     pub name: String,
///     pub count: i64,
/// }
/// ```
/// 
/// - `Queryable`: Automatically generates SQL SELECT statements
/// - `SqlParams`: Automatically generates SQL parameters
/// - `#[table("table_name")]`: Specifies the table name for the query
/// - `#[where_clause("id = $")]`: Specifies the query condition (`$` will be replaced with parameter value)
/// 
/// ## Example Usage
/// ```rust,no_run
/// use postgres::{Client, NoTls, Error};
/// use parsql::postgres::select;
/// 
/// #[derive(Queryable, SqlParams)]
/// #[table("users")]
/// #[where_clause("id = $")]
/// pub struct UserQuery {
///     pub id: i32,
/// }
/// 
/// impl UserQuery {
///     pub fn new(id: i32) -> Self {
///         Self { id }
///     }
/// }
/// 
/// // Different return structure
/// pub struct User {
///     pub id: i32,
///     pub name: String,
/// }
///
/// fn main() -> Result<(), Error> {
///     let mut client = Client::connect(
///         "host=localhost user=postgres dbname=test",
///         NoTls,
///     )?;
///
///     // A custom model transformation function is required
///     let user_query = UserQuery::new(1);
///     let user = select(&mut client, user_query, |row| {
///         let id: i32 = row.get("id");
///         let name: String = row.get("name");
///         Ok(User { id, name })
///     })?;
///     
///     println!("User: {:?}", user);
///     Ok(())
/// }
/// ```
pub fn select<T: SqlQuery + SqlParams, F>(
    client: &mut postgres::Client,
    entity: T,
    to_model: F,
) -> Result<T, Error>
where
    F: Fn(&Row) -> Result<T, Error>,
{
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-POSTGRES] Execute SQL: {}", sql);
    }

    let params = entity.params();

    match client.query_one(&sql, &params) {
        Ok(_row) => to_model(&_row),
        Err(e) => Err(e),
    }
}

/// # select_all
/// 
/// Retrieves multiple records from the database using a custom transformation function.
/// This is useful when you want to use a custom transformation function instead of the FromRow trait.
/// 
/// ## Parameters
/// - `client`: Database connection client
/// - `entity`: Query parameter object (must implement SqlQuery and SqlParams traits)
/// - `to_model`: Function to convert a Row object to the target object type
/// 
/// ## Return Value
/// - `Result<Vec<T>, Error>`: On success, returns the list of transformed objects; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust,no_run
/// #[derive(Queryable, SqlParams)]          // Required macros (FromRow is not needed)
/// #[table("table_name")]                   // Table name to query
/// #[select("id, name, COUNT(*) as count")] // Custom SELECT statement (optional)
/// #[where_clause("active = $")]            // Query condition
/// pub struct MyQueryEntity {
///     pub active: bool,                    // Field used in the query condition
///     // Other fields can be added if necessary for the query condition
/// }
/// 
/// // A separate struct can be used for the return value
/// pub struct MyResultEntity {
///     pub id: i32,
///     pub name: String,
///     pub count: i64,
/// }
/// ```
/// 
/// - `Queryable`: Automatically generates SQL SELECT statements
/// - `SqlParams`: Automatically generates SQL parameters
/// - `#[table("table_name")]`: Specifies the table name for the query
/// - `#[select("...")]`: Creates a custom SELECT statement (if omitted, all fields will be selected)
/// - `#[where_clause("active = $")]`: Specifies the query condition (`$` will be replaced with parameter value)
/// 
/// ## Example Usage
/// ```rust,no_run
/// use postgres::{Client, NoTls, Error};
/// use parsql::postgres::select_all;
/// 
/// #[derive(Queryable, SqlParams)]
/// #[table("users")]
/// #[select("id, name, email")]
/// pub struct UsersQuery {
///     // Can be empty for a parameterless query
/// }
/// 
/// impl UsersQuery {
///     pub fn new() -> Self {
///         Self {}
///     }
/// }
/// 
/// // Different return structure
/// pub struct User {
///     pub id: i32,
///     pub name: String,
/// }
///
/// fn main() -> Result<(), Error> {
///     let mut client = Client::connect(
///         "host=localhost user=postgres dbname=test",
///         NoTls,
///     )?;
///
///     // A custom model transformation function is required
///     let users_query = UsersQuery::new();
///     let users = select_all(&mut client, users_query, |row| {
///         let id: i32 = row.get("id");
///         let name: String = row.get("name");
///         User { id, name }
///     })?;
///     
///     println!("Users: {:?}", users);
///     Ok(())
/// }
/// ```
pub fn select_all<T: SqlQuery + SqlParams, F>(
    client: &mut postgres::Client,
    entity: T,
    to_model: F,
) -> Result<Vec<T>, Error>
where
    F: Fn(&Row) -> Result<T, Error>,
{
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-POSTGRES] Execute SQL: {}", sql);
    }

    let params = entity.params();

    let rows = client.query(&sql, &params)?;

    rows.iter()
        .map(|row| to_model(row))
        .collect::<Result<Vec<_>, _>>()
}
