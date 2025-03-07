use rusqlite::{Error, Row, ToSql};
use parsql_core::{SqlQuery, SqlParams, UpdateParams, FromRow};

/// # insert
/// 
/// Inserts a new record into the SQLite database.
/// 
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Data object to be inserted (must implement SqlQuery and SqlParams traits)
/// 
/// ## Return Value
/// - `Result<usize, rusqlite::Error>`: On success, returns the number of inserted records; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust
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
/// ```rust
/// // Define an entity for insertion
/// #[derive(Insertable, SqlParams)]
/// #[table("users")]
/// pub struct InsertUser {
///     pub name: String,
///     pub email: String,
///     pub state: i16,
/// }
///
/// // Create a new user
/// let insert_user = InsertUser {
///     name: "John".to_string(),
///     email: "john@example.com".to_string(),
///     state: 1,
/// };
///
/// // Insert into database
/// let insert_result = insert(&conn, insert_user);
/// println!("Insert result: {:?}", insert_result);
/// ```
pub fn insert<T: SqlQuery + SqlParams>(
    conn: &rusqlite::Connection,
    entity: T,
) -> Result<usize, rusqlite::Error> {
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
    }

    let _params: Vec<&dyn ToSql> = entity.params().iter().map(|p| *p as &dyn ToSql).collect();

    match conn.execute(&sql, _params.as_slice()) {
        Ok(_result) => Ok(_result),
        Err(_err) => panic!("Insert işlemi yürütme esnasında bir hata oluştu! {}", _err),
    }
}

/// # update
/// 
/// Updates an existing record in the SQLite database.
/// 
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Data object containing the update information (must implement SqlQuery and UpdateParams traits)
/// 
/// ## Return Value
/// - `Result<usize, Error>`: On success, returns the number of updated records; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust
/// #[derive(Updateable, UpdateParams)]  // Required macros
/// #[table("table_name")]               // Table name to update
/// #[update("field1, field2")]          // Fields to update (optional)
/// #[where_clause("id = $")]            // Update condition
/// pub struct MyEntity {
///     pub id: i64,                     // Fields used in the condition
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
/// ```rust
/// // Define an entity for updating
/// #[derive(Updateable, UpdateParams)]
/// #[table("users")]
/// #[update("name, email")]
/// #[where_clause("id = $")]
/// pub struct UpdateUser {
///     pub id: i64,
///     pub name: String,
///     pub email: String,
///     pub state: i16,  // This field won't be updated as it's not specified in the update attribute
/// }
///
/// // Create update data
/// let update_user = UpdateUser {
///     id: 1,
///     name: String::from("John"),
///     email: String::from("john@example.com"),
///     state: 2,
/// };
///
/// // Update in database
/// let update_result = update(&conn, update_user);
/// println!("Update result: {:?}", update_result);
/// ```
pub fn update<T: SqlQuery + UpdateParams>(
    conn: &rusqlite::Connection,
    entity: T,
) -> Result<usize, Error> {
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
    }

    let _params: Vec<&dyn ToSql> = entity.params().iter().map(|p| *p as &dyn ToSql).collect();

    match conn.execute(&sql, _params.as_slice()) {
        Ok(rows_affected) => Ok(rows_affected),
        Err(e) => Err(e),
    }
}

/// # delete
/// 
/// Deletes a record from the SQLite database.
/// 
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Data object containing the deletion information (must implement SqlQuery and SqlParams traits)
/// 
/// ## Return Value
/// - `Result<usize, Error>`: On success, returns the number of deleted records; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust
/// #[derive(Deleteable, SqlParams)]   // Required macros
/// #[table("table_name")]             // Table name to delete from
/// #[where_clause("id = $")]          // Delete condition
/// pub struct MyEntity {
///     pub id: i64,                   // Fields used in the condition
///     // Other fields can be added, but typically only condition fields are necessary
/// }
/// ```
/// 
/// - `Deleteable`: Automatically generates SQL DELETE statements
/// - `SqlParams`: Automatically generates SQL parameters
/// - `#[table("table_name")]`: Specifies the table name for the deletion
/// - `#[where_clause("id = $")]`: Specifies the delete condition (`$` will be replaced with parameter value)
/// 
/// ## Example Usage
/// ```rust
/// // Define an entity for deletion
/// #[derive(Deleteable, SqlParams)]
/// #[table("users")]
/// #[where_clause("id = $")]
/// pub struct DeleteUser {
///     pub id: i64,
/// }
/// 
/// // Create delete data
/// let delete_user = DeleteUser { id: 6 };
/// 
/// // Delete from database
/// let delete_result = delete(&conn, delete_user);
/// println!("Delete result: {:?}", delete_result);
/// ```
pub fn delete<T: SqlQuery + SqlParams>(
    conn: &rusqlite::Connection,
    entity: T,
) -> Result<usize, Error> {
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
    }

    let _params: Vec<&dyn ToSql> = entity.params().iter().map(|p| *p as &dyn ToSql).collect();

    match conn.execute(&sql, _params.as_slice()) {
        Ok(rows_affected) => Ok(rows_affected),
        Err(e) => Err(e),
    }
}

/// # get
/// 
/// Retrieves a single record from the SQLite database.
/// 
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Query parameter object (must implement SqlQuery, FromRow, and SqlParams traits)
/// 
/// ## Return Value
/// - `Result<T, Error>`: On success, returns the found record; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust
/// #[derive(Queryable, FromRow, SqlParams, Debug)]  // Required macros
/// #[table("table_name")]                           // Table name to query
/// #[where_clause("id = $")]                        // Query condition
/// pub struct MyEntity {
///     pub id: i64,                                 // Field used in the query condition
///     pub field1: String,                          // Fields to be populated from the result set
///     pub field2: i32,
///     // ...
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
/// ```rust
/// // Define a query entity
/// #[derive(Queryable, FromRow, SqlParams, Debug)]
/// #[table("users")]
/// #[where_clause("id = $")]
/// pub struct GetUser {
///     pub id: i64,
///     pub name: String,
///     pub email: String,
///     pub state: i16,
/// }
///
/// // Create query parameters
/// let get_user = GetUser {
///     id: 1,
///     name: Default::default(),
///     email: Default::default(),
///     state: Default::default(),
/// };
/// 
/// // Retrieve from database
/// let get_result = get(&conn, get_user);
/// println!("Get result: {:?}", get_result);
/// ```
/// 
/// ## Security
/// This function is protected against SQL injection as all parameters are properly escaped.
/// 
/// ```rust
/// // Example with a potentially malicious input
/// let malicious_name = "' OR '1'='1";
/// 
/// #[derive(Queryable, FromRow, SqlParams, Debug)]
/// #[table("users")]
/// #[where_clause("name = $")]
/// pub struct GetUserByName {
///     pub id: i64,
///     pub name: String,
///     pub email: String,
///     pub state: i16,
/// }
/// 
/// let get_user = GetUserByName {
///     id: 0,
///     name: malicious_name.to_string(),
///     email: String::new(),
///     state: 0,
/// };
/// 
/// // This is safe and will only look for a user with the literal name "' OR '1'='1"
/// match get(&conn, get_user) {
///     Ok(user) => println!("Found user: {:?}", user),
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
pub fn get<T: SqlQuery + FromRow + SqlParams>(
    conn: &rusqlite::Connection,
    entity: T,
) -> Result<T, Error> {
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
    }

    let _params: Vec<&dyn ToSql> = entity.params().iter().map(|p| *p as &dyn ToSql).collect();

    conn.query_row(&sql, _params.as_slice(), |row| T::from_row(row))
}

/// # get_all
/// 
/// Retrieves multiple records from the SQLite database.
/// 
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Query parameter object (must implement SqlQuery, FromRow, and SqlParams traits)
/// 
/// ## Return Value
/// - `Result<Vec<T>, Error>`: On success, returns the list of found records; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust
/// #[derive(Queryable, FromRow, SqlParams, Debug)]  // Required macros
/// #[table("table_name")]                           // Table name to query
/// #[select("field1, field2, COUNT(*) as count")]   // Custom SELECT statement (optional)
/// #[join("LEFT JOIN other_table ON ...")]          // JOIN statements (optional)
/// #[where_clause("status > $")]                    // Query condition
/// #[group_by("field1, field2")]                    // GROUP BY statement (optional)
/// #[having("COUNT(*) > 0")]                        // HAVING statement (optional)
/// #[order_by("count DESC")]                        // ORDER BY statement (optional)
/// pub struct MyEntity {
///     pub status: i16,                             // Field used in the query condition
///     pub field1: String,                          // Fields to be populated from the result set
///     pub field2: i32,
///     pub count: i64,                              // Calculated value
///     // ...
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
/// ```rust
/// // Advanced example with GROUP BY, HAVING, and JOIN
/// #[derive(Queryable, FromRow, SqlParams, Debug)]
/// #[table("users")]
/// #[select("users.state, posts.state as post_state, COUNT(posts.id) as post_count, AVG(CAST(posts.id as REAL)) as avg_post_id")]
/// #[join("LEFT JOIN posts ON users.id = posts.user_id")]
/// #[where_clause("users.state > $")]
/// #[group_by("users.state, posts.state")]
/// #[having("COUNT(posts.id) > 0 AND AVG(CAST(posts.id as REAL)) > 2")]
/// #[order_by("post_count DESC")]
/// pub struct UserPostStatsAdvanced {
///     pub state: i16,
///     pub post_state: Option<i16>,
///     pub post_count: i64,
///     pub avg_post_id: Option<f32>,
/// }
///
/// impl UserPostStatsAdvanced {
///     pub fn new(min_state: i16) -> Self {
///         Self {
///             state: min_state,
///             post_state: None,
///             post_count: 0,
///             avg_post_id: None,
///         }
///     }
/// }
///
/// // Create a query for user-post statistics
/// let stats_query = UserPostStatsAdvanced::new(0);
/// 
/// // Get all statistics
/// match get_all(&conn, stats_query) {
///     Ok(stats) => {
///         println!("User-post statistics:");
///         for stat in stats {
///             println!("  {:?}", stat);
///         }
///     },
///     Err(e) => println!("Error retrieving statistics: {}", e),
/// }
/// ```
pub fn get_all<T: SqlQuery + FromRow + SqlParams>(
    conn: &rusqlite::Connection,
    entity: T,
) -> Result<Vec<T>, Error> {
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
    }
    let _params: Vec<&dyn ToSql> = entity.params().iter().map(|p| *p as &dyn ToSql).collect();
    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(_params.as_slice(), |row| T::from_row(row))?;
    let results = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(results)
}

/// # select
/// 
/// Retrieves a single record from the SQLite database using a custom transformation function.
/// This is useful when you want to use a custom transformation function instead of the FromRow trait.
/// 
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Query parameter object (must implement SqlQuery and SqlParams traits)
/// - `to_model`: Function to convert a Row object to the target object type
/// 
/// ## Return Value
/// - `Result<T, Error>`: On success, returns the transformed object; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust
/// #[derive(Queryable, SqlParams)]          // Required macros (FromRow is not needed)
/// #[table("table_name")]                   // Table name to query
/// #[where_clause("id = $")]                // Query condition
/// pub struct MyQueryEntity {
///     pub id: i64,                         // Field used in the query condition
///     // Other fields can be added if necessary for the query condition
/// }
/// ```
/// 
/// ## Example Usage
/// ```rust
/// // Define a query entity
/// #[derive(Queryable, SqlParams)]
/// #[table("users")]
/// #[where_clause("id = $")]
/// pub struct UserQuery {
///     pub id: i64,
/// }
///
/// // Define a result entity
/// #[derive(Debug)]
/// pub struct User {
///     pub id: i64,
///     pub name: String,
///     pub email: String,
///     pub state: i16,
/// }
///
/// // Create query parameters
/// let user_query = UserQuery { id: 1 };
///
/// // Execute query with custom transformation
/// let select_result = select(&mut conn, user_query, |row| {
///     Ok(User {
///         id: row.get(0)?,
///         name: row.get(1)?,
///         email: row.get(2)?,
///         state: row.get(3)?,
///     })
/// });
///
/// println!("Select result: {:?}", select_result);
/// ```
pub fn select<T: SqlQuery + SqlParams, F>(
    conn: &mut rusqlite::Connection,
    entity: T,
    to_model: F,
) -> Result<T, Error>
where
    F: Fn(&Row) -> Result<T, Error>,
{

    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
    }

    let params: Vec<&dyn ToSql> = entity.params().iter().map(|p| *p as &dyn ToSql).collect();

    match conn.query_row(&sql, params.as_slice(), |row| to_model(row)) {
        Ok(row) => Ok(row),
        Err(e) => Err(e),
    }
}

/// # select_all
/// 
/// Retrieves multiple records from the SQLite database using a custom transformation function.
/// This is useful when you want to use a custom transformation function instead of the FromRow trait.
/// 
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Query parameter object (must implement SqlQuery and SqlParams traits)
/// - `to_model`: Function to convert a Row object to the target object type
/// 
/// ## Return Value
/// - `Result<Vec<T>, Error>`: On success, returns the list of transformed objects; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust
/// #[derive(Queryable, SqlParams)]          // Required macros (FromRow is not needed)
/// #[table("table_name")]                   // Table name to query
/// #[select("id, name, COUNT(*) as count")] // Custom SELECT statement (optional)
/// #[where_clause("state > $")]             // Query condition
/// pub struct MyQueryEntity {
///     pub state: i16,                      // Field used in the query condition
///     // Other fields can be added if necessary for the query condition
/// }
/// ```
/// 
/// ## Example Usage
/// ```rust
/// // Define a query entity
/// #[derive(Queryable, SqlParams)]
/// #[table("users")]
/// #[select("id, name, email, state")]
/// #[where_clause("state > $")]
/// pub struct UsersQuery {
///     pub min_state: i16,
/// }
/// 
/// // Define a result entity
/// #[derive(Debug)]
/// pub struct User {
///     pub id: i64,
///     pub name: String,
///     pub email: String,
///     pub state: i16,
/// }
///
/// // Create query parameters
/// let users_query = UsersQuery { min_state: 0 };
/// 
/// // Execute query with custom transformation
/// let users = select_all(&mut conn, users_query, |row| {
///     Ok(User {
///         id: row.get(0)?,
///         name: row.get(1)?,
///         email: row.get(2)?,
///         state: row.get(3)?,
///     })
/// });
///
/// match users {
///     Ok(users) => {
///         println!("Users with state > 0:");
///         for user in users {
///             println!("  {:?}", user);
///         }
///     },
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
pub fn select_all<T: SqlQuery + SqlParams, F>(
    conn: &mut rusqlite::Connection,
    entity: T,
    to_model: F,
) -> Result<Vec<T>, Error>
where
    F: Fn(&Row) -> Result<T, Error>,
{
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
    }

    let params: Vec<&dyn ToSql> = entity.params().iter().map(|p| *p as &dyn ToSql).collect();

    let mut stmt = conn.prepare(&sql).unwrap();

    stmt.query_map(params.as_slice(), |row| to_model(row))
        .map(|iter| iter.collect::<Result<Vec<T>, _>>())
        .map_err(|err| println!("{:?}", err))
        .unwrap()
}
