use rusqlite::{Error, Row, ToSql};
use crate::{SqlQuery, SqlParams, UpdateParams, FromRow};
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
/// 
/// ```rust,ignore
/// // Required imports
/// use rusqlite::Connection;
/// use parsql_macros::{Insertable, SqlParams};
/// use parsql_sqlite::insert;
/// 
/// // Create a database connection
/// let conn = Connection::open_in_memory().unwrap();
/// 
/// // Create the table
/// conn.execute("CREATE TABLE users (name TEXT, email TEXT, state INTEGER)", []).unwrap();
/// 
/// // Define your entity with appropriate macros
/// #[derive(Insertable, SqlParams)]
/// #[table("users")]
/// pub struct InsertUser {
///     pub name: String,
///     pub email: String,
///     pub state: i16,
/// }
///
/// // Create a new instance of your entity
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
/// Updates a record in the database.
/// 
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: The entity to update (must implement SqlQuery and UpdateParams traits)
/// 
/// ## Return Value
/// - `Result<usize, Error>`: On success, returns the number of rows affected; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust,ignore
/// use parsql_macros::{Updateable, UpdateParams};
/// 
/// #[derive(Updateable, UpdateParams)]  // Required macros
/// #[table("table_name")]              // Table name to update
/// #[update("field1, field2")]         // Fields to update
/// #[where_clause("id = $")]           // Update condition
/// pub struct MyEntity {
///     pub id: i64,                    // Field used in the where clause
///     pub field1: String,             // Fields to update
///     pub field2: i32,
/// }
/// ```
/// 
/// ## Example Usage
/// 
/// ```rust,ignore
/// use rusqlite::Connection;
/// use parsql_macros::{Updateable, UpdateParams};
/// use parsql_sqlite::update;
/// 
/// // Create database connection
/// let conn = Connection::open_in_memory().unwrap();
/// conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT, state INTEGER)", []).unwrap();
/// conn.execute("INSERT INTO users (id, name, email, state) VALUES (1, 'Old Name', 'old@example.com', 0)", []).unwrap();
/// 
/// // Define an entity for updating
/// #[derive(Updateable, UpdateParams)]
/// #[table("users")]
/// #[update("name, email")]
/// #[where_clause("id = $")]
/// pub struct UpdateUser {
///     pub id: i64,
///     pub name: String,
///     pub email: String,
///     pub state: i16,  // Not included in the update
/// }
/// 
/// // Create an update object
/// let update_user = UpdateUser {
///     id: 1,  // User ID to update
///     name: "New Name".to_string(),
///     email: "new@example.com".to_string(),
///     state: 1,
/// };
/// 
/// // Execute the update
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
/// Deletes records from the database based on a specific condition.
/// 
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Query parameter object (must implement SqlQuery and SqlParams traits)
/// 
/// ## Return Value
/// - `Result<usize, Error>`: On success, returns the number of deleted records; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust,ignore
/// use parsql_macros::{Queryable, SqlParams};
/// 
/// #[derive(Queryable, SqlParams)]  // Required macros
/// #[table("table_name")]           // Table name to delete from
/// #[where_clause("id = $")]        // Delete condition
/// pub struct MyEntity {
///     pub id: i64,                 // Field used in the condition
/// }
/// ```
/// 
/// ## Example Usage
/// 
/// ```rust,ignore
/// use rusqlite::Connection;
/// use parsql_macros::{Queryable, SqlParams};
/// use parsql_sqlite::delete;
/// 
/// // Create database connection
/// let conn = Connection::open_in_memory().unwrap();
/// conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)", []).unwrap();
/// conn.execute("INSERT INTO users (id, name, email) VALUES (1, 'John', 'john@example.com')", []).unwrap();
/// 
/// // Define a delete query
/// #[derive(Queryable, SqlParams)]
/// #[table("users")]
/// #[where_clause("id = $")]
/// pub struct DeleteUser {
///     pub id: i64,
/// }
/// 
/// // Create delete parameters (delete user with ID 1)
/// let delete_query = DeleteUser { id: 1 };
/// 
/// // Execute delete
/// let delete_result = delete(&conn, delete_query);
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
/// Retrieves a single record from the database based on a specific condition.
/// 
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Query parameter object (must implement SqlQuery, FromRow, and SqlParams traits)
/// 
/// ## Return Value
/// - `Result<T, Error>`: On success, returns the retrieved record; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust,ignore
/// use parsql_macros::{Queryable, FromRow, SqlParams};
/// 
/// #[derive(Queryable, FromRow, SqlParams)]  // Required macros
/// #[table("table_name")]                    // Table name to query
/// #[select("field1, field2, field3")]       // Fields to select (optional)
/// #[where_clause("id = $")]                 // Query condition
/// pub struct MyEntity {
///     pub id: i64,                          // Field used in the condition
///     pub field1: String,                   // Fields to retrieve
///     pub field2: i32,
///     pub field3: Option<String>,
/// }
/// ```
/// 
/// ## Example Usage
/// 
/// ```rust,ignore
/// use rusqlite::Connection;
/// use parsql_macros::{Queryable, FromRow, SqlParams};
/// use parsql_sqlite::get;
/// 
/// // Create database connection
/// let conn = Connection::open_in_memory().unwrap();
/// conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT, state INTEGER)", []).unwrap();
/// conn.execute("INSERT INTO users (id, name, email, state) VALUES (1, 'John', 'john@example.com', 1)", []).unwrap();
/// 
/// // Define a query entity
/// #[derive(Queryable, FromRow, SqlParams)]
/// #[table("users")]
/// #[where_clause("id = $")]
/// pub struct GetUser {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
///     pub state: i16,
/// }
/// 
/// // Create query parameters (get user with ID 1)
/// let get_query = GetUser {
///     id: 1,
///     name: String::new(),
///     email: String::new(),
///     state: 0,
/// };
/// 
/// // Execute query
/// match get(&conn, get_query) {
///     Ok(user) => println!("Retrieved user: {} ({})", user.name, user.email),
///     Err(e) => println!("Error retrieving user: {}", e),
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
/// Retrieves multiple records from the database based on a specific condition.
/// 
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Query parameter object (must implement SqlQuery, FromRow, and SqlParams traits)
/// 
/// ## Return Value
/// - `Result<Vec<T>, Error>`: On success, returns a vector of retrieved records; on failure, returns Error
/// 
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
/// 
/// ```rust,no_run
/// # use parsql_macros::{Queryable, FromRow, SqlParams};
/// #[derive(Queryable, FromRow, SqlParams)]  // Required macros
/// #[table("table_name")]                    // Table name to query
/// #[select("field1, field2, field3")]       // Fields to select (optional)
/// #[where_clause("status = $")]             // Query condition
/// #[order_by("field1 DESC")]                // Order By clause (optional)
/// pub struct MyEntity {
///     pub status: i32,                      // Field used in the condition
///     pub field1: String,                   // Fields to retrieve
///     pub field2: i32,
///     pub field3: Option<String>,
/// }
/// ```
/// 
/// ## Example Usage
/// 
/// ```rust,no_run
/// # use rusqlite::Connection;
/// # use parsql_macros::{Queryable, FromRow, SqlParams};
/// # use parsql_sqlite::get_all;
/// # 
/// # // Create database connection
/// # let conn = Connection::open_in_memory().unwrap();
/// # conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT, state INTEGER)", []).unwrap();
/// # conn.execute("INSERT INTO users (id, name, email, state) VALUES (1, 'John', 'john@example.com', 1)", []).unwrap();
/// # conn.execute("INSERT INTO users (id, name, email, state) VALUES (2, 'Jane', 'jane@example.com', 1)", []).unwrap();
/// # 
/// // Define a query entity for retrieving all active users
/// #[derive(Queryable, FromRow, SqlParams)]
/// #[table("users")]
/// #[where_clause("state = $")]
/// #[order_by("name ASC")]
/// pub struct GetActiveUsers {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
///     pub state: i16,
/// }
/// 
/// // Create query parameters to get all users with state = 1
/// let query = GetActiveUsers {
///     id: 0,
///     name: String::new(),
///     email: String::new(),
///     state: 1,  // Active state
/// };
/// 
/// // Execute query to get all matching records
/// match get_all(&conn, query) {
///     Ok(users) => {
///         println!("Found {} active users:", users.len());
///         for user in users {
///             println!("- {} ({})", user.name, user.email);
///         }
///     },
///     Err(e) => println!("Error retrieving users: {}", e),
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
