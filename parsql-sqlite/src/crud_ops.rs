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
/// 
/// ```rust,no_run
/// use rusqlite::{Connection, Result};
/// use parsql_macros::{Insertable, SqlParams};
/// use parsql_sqlite::insert;
/// 
/// fn main() -> Result<()> {
///     // Create a database connection
///     let conn = Connection::open("test.db")?;
/// 
///     // Create the table
///     conn.execute("CREATE TABLE users (name TEXT, email TEXT, state INTEGER)", [])?;
/// 
///     // Define your entity with appropriate macros
///     #[derive(Insertable, SqlParams)]
///     #[table("users")]
///     pub struct InsertUser {
///         pub name: String,
///         pub email: String,
///         pub state: i16,
///     }
///
///     // Create a new instance of your entity
///     let insert_user = InsertUser {
///         name: "John".to_string(),
///         email: "john@example.com".to_string(),
///         state: 1,
///     };
///
///     // Insert into database
///     let insert_result = insert(&conn, insert_user)?;
///     println!("Insert result: {:?}", insert_result);
///     Ok(())
/// }
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
        Err(e) => Err(e),
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
/// ```rust,no_run
/// use parsql_macros::{Updateable, UpdateParams};
/// 
/// #[derive(Updateable, UpdateParams)]  // Required macros
/// #[table("table_name")]              // Table name to update
/// #[update("field1, field2")]         // Fields to update
/// #[where_clause("id = ?")]           // Update condition
/// pub struct MyEntity {
///     pub id: i64,                    // Field used in the where clause
///     pub field1: String,             // Fields to update
///     pub field2: i32,
/// }
/// ```
/// 
/// ## Example Usage
/// 
/// ```rust,no_run
/// use rusqlite::{Connection, Result};
/// use parsql_macros::{Updateable, UpdateParams};
/// use parsql_sqlite::update;
/// 
/// fn main() -> Result<()> {
///     // Create database connection
///     let conn = Connection::open("test.db")?;
///     conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT, state INTEGER)", [])?;
///     conn.execute("INSERT INTO users (id, name, email, state) VALUES (1, 'Old Name', 'old@example.com', 0)", [])?;
/// 
///     // Define an entity for updating
///     #[derive(Updateable, UpdateParams)]
///     #[table("users")]
///     #[update("name, email")]
///     #[where_clause("id = ?")]
///     pub struct UpdateUser {
///         pub id: i64,
///         pub name: String,
///         pub email: String,
///         pub state: i16,  // Not included in the update
///     }
/// 
///     // Create an update object
///     let update_user = UpdateUser {
///         id: 1,  // User ID to update
///         name: "New Name".to_string(),
///         email: "new@example.com".to_string(),
///         state: 1,
///     };
/// 
///     // Execute the update
///     let update_result = update(&conn, update_user)?;
///     println!("Update result: {:?}", update_result);
///     Ok(())
/// }
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
/// ```rust,no_run
/// use parsql_macros::{Deletable, SqlParams};
/// 
/// #[derive(Deletable, SqlParams)]  // Required macros
/// #[table("table_name")]           // Table name to delete from
/// #[where_clause("id = ?")]        // Delete condition
/// pub struct MyEntity {
///     pub id: i64,                 // Field used in the condition
/// }
/// ```
/// 
/// ## Example Usage
/// 
/// ```rust,no_run
/// use rusqlite::{Connection, Result};
/// use parsql_macros::{Deletable, SqlParams};
/// use parsql_sqlite::delete;
/// 
/// fn main() -> Result<()> {
///     // Create database connection
///     let conn = Connection::open("test.db")?;
///     conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)", [])?;
///     conn.execute("INSERT INTO users (id, name, email) VALUES (1, 'John', 'john@example.com')", [])?;
/// 
///     // Define a delete query
///     #[derive(Deletable, SqlParams)]
///     #[table("users")]
///     #[where_clause("id = ?")]
///     pub struct DeleteUser {
///         pub id: i64,
///     }
/// 
///     // Create delete parameters (delete user with ID 1)
///     let delete_query = DeleteUser { id: 1 };
/// 
///     // Execute delete
///     let delete_result = delete(&conn, delete_query)?;
///     println!("Delete result: {:?}", delete_result);
///     Ok(())
/// }
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
/// ```rust,no_run
/// use parsql_macros::{Queryable, FromRow, SqlParams};
/// 
/// #[derive(Queryable, FromRow, SqlParams)]  // Required macros
/// #[table("table_name")]                    // Table name to query
/// #[where_clause("id = ?")]                 // Query condition
/// pub struct MyEntity {
///     pub id: i64,                          // Field used in the condition
///     pub field1: String,                   // Fields to retrieve
///     pub field2: i32,
/// }
/// ```
/// 
/// ## Example Usage
/// 
/// ```rust,no_run
/// use rusqlite::{Connection, Result};
/// use parsql_macros::{Queryable, FromRow, SqlParams};
/// use parsql_sqlite::get;
/// 
/// fn main() -> Result<()> {
///     // Create database connection
///     let conn = Connection::open("test.db")?;
///     conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)", [])?;
///     conn.execute("INSERT INTO users (id, name, email) VALUES (1, 'John', 'john@example.com')", [])?;
/// 
///     // Define a query
///     #[derive(Queryable, FromRow, SqlParams)]
///     #[table("users")]
///     #[where_clause("id = ?")]
///     pub struct GetUser {
///         pub id: i64,
///         pub name: String,
///         pub email: String,
///     }
/// 
///     // Create query parameters (get user with ID 1)
///     let get_query = GetUser {
///         id: 1,
///         name: String::new(),
///         email: String::new(),
///     };
/// 
///     // Execute query
///     let user = get(&conn, get_query)?;
///     println!("User: {:?}", user);
///     Ok(())
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
/// use parsql_macros::{Queryable, FromRow, SqlParams};
/// 
/// #[derive(Queryable, FromRow, SqlParams)]  // Required macros
/// #[table("table_name")]                    // Table name to query
/// #[where_clause("state = ?")]              // Query condition
/// pub struct MyEntity {
///     pub state: i16,                       // Field used in the condition
///     pub field1: String,                   // Fields to retrieve
///     pub field2: i32,
/// }
/// ```
/// 
/// ## Example Usage
/// 
/// ```rust,no_run
/// use rusqlite::{Connection, Result};
/// use parsql_macros::{Queryable, FromRow, SqlParams};
/// use parsql_sqlite::get_all;
/// 
/// fn main() -> Result<()> {
///     // Create database connection
///     let conn = Connection::open("test.db")?;
///     conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT, state INTEGER)", [])?;
///     conn.execute("INSERT INTO users (name, email, state) VALUES ('John', 'john@example.com', 1)", [])?;
///     conn.execute("INSERT INTO users (name, email, state) VALUES ('Jane', 'jane@example.com', 1)", [])?;
/// 
///     // Define a query
///     #[derive(Queryable, FromRow, SqlParams)]
///     #[table("users")]
///     #[where_clause("state = ?")]
///     pub struct GetActiveUsers {
///         pub id: i64,
///         pub name: String,
///         pub email: String,
///         pub state: i16,
///     }
/// 
///     // Create query parameters (get all active users)
///     let get_query = GetActiveUsers {
///         id: 0,
///         name: String::new(),
///         email: String::new(),
///         state: 1,
///     };
/// 
///     // Execute query
///     let users = get_all(&conn, get_query)?;
///     println!("Active users: {:?}", users);
///     Ok(())
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
/// Executes a custom SELECT query and maps the result to a model using a provided mapping function.
/// 
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Query parameter object (must implement SqlQuery and SqlParams traits)
/// - `to_model`: Function to map a database row to your model type
/// 
/// ## Return Value
/// - `Result<T, Error>`: On success, returns the mapped model; on failure, returns Error
/// 
/// ## Example Usage
/// 
/// ```rust,no_run
/// use rusqlite::{Connection, Result, Row};
/// use parsql_macros::{Queryable, SqlParams};
/// use parsql_sqlite::select;
/// 
/// fn main() -> Result<()> {
///     // Create database connection
///     let conn = Connection::open("test.db")?;
///     conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)", [])?;
///     conn.execute("INSERT INTO users (id, name, email) VALUES (1, 'John', 'john@example.com')", [])?;
/// 
///     // Define your model
///     #[derive(Debug)]
///     pub struct User {
///         pub id: i64,
///         pub name: String,
///         pub email: String,
///     }
/// 
///     // Define a query
///     #[derive(Queryable, SqlParams)]
///     #[table("users")]
///     #[where_clause("id = ?")]
///     pub struct GetUser {
///         pub id: i64,
///     }
/// 
///     // Create query parameters
///     let get_query = GetUser { id: 1 };
/// 
///     // Define row mapping function
///     let to_user = |row: &Row| -> Result<User> {
///         Ok(User {
///             id: row.get(0)?,
///             name: row.get(1)?,
///             email: row.get(2)?,
///         })
///     };
/// 
///     // Execute query with custom mapping
///     let user = select(&mut conn, get_query, to_user)?;
///     println!("User: {:?}", user);
///     Ok(())
/// }
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
/// Executes a custom SELECT query and maps multiple results to models using a provided mapping function.
/// 
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Query parameter object (must implement SqlQuery and SqlParams traits)
/// - `to_model`: Function to map a database row to your model type
/// 
/// ## Return Value
/// - `Result<Vec<T>, Error>`: On success, returns a vector of mapped models; on failure, returns Error
/// 
/// ## Example Usage
/// 
/// ```rust,no_run
/// use rusqlite::{Connection, Result, Row};
/// use parsql_macros::{Queryable, SqlParams};
/// use parsql_sqlite::select_all;
/// 
/// fn main() -> Result<()> {
///     // Create database connection
///     let conn = Connection::open("test.db")?;
///     conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT, state INTEGER)", [])?;
///     conn.execute("INSERT INTO users (name, email, state) VALUES ('John', 'john@example.com', 1)", [])?;
///     conn.execute("INSERT INTO users (name, email, state) VALUES ('Jane', 'jane@example.com', 1)", [])?;
/// 
///     // Define your model
///     #[derive(Debug)]
///     pub struct User {
///         pub id: i64,
///         pub name: String,
///         pub email: String,
///         pub state: i16,
///     }
/// 
///     // Define a query
///     #[derive(Queryable, SqlParams)]
///     #[table("users")]
///     #[where_clause("state = ?")]
///     pub struct GetActiveUsers {
///         pub state: i16,
///     }
/// 
///     // Create query parameters
///     let get_query = GetActiveUsers { state: 1 };
/// 
///     // Define row mapping function
///     let to_user = |row: &Row| -> Result<User> {
///         Ok(User {
///             id: row.get(0)?,
///             name: row.get(1)?,
///             email: row.get(2)?,
///             state: row.get(3)?,
///         })
///     };
/// 
///     // Execute query with custom mapping
///     let users = select_all(&mut conn, get_query, to_user)?;
///     println!("Active users: {:?}", users);
///     Ok(())
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
