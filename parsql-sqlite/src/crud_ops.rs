use rusqlite::{types::FromSql, Error, Row, ToSql};

use crate::traits::{CrudOps, FromRow, SqlCommand, SqlParams, SqlQuery, UpdateParams};

// CrudOps trait implementasyonu rusqlite::Connection için
impl CrudOps for rusqlite::Connection {
    fn insert<T: SqlCommand + SqlParams, P: for<'a> FromSql + Send + Sync>(
        &self,
        entity: T,
    ) -> Result<P, Error> {
        insert(self, entity)
    }

    fn update<T: SqlCommand + UpdateParams>(&self, entity: T) -> Result<usize, Error> {
        update(self, entity)
    }

    fn delete<T: SqlCommand + SqlParams>(&self, entity: T) -> Result<usize, Error> {
        delete(self, entity)
    }

    fn fetch<P, R>(&self, params: &P) -> Result<R, Error>
    where
        P: SqlQuery<R> + SqlParams,
        R: FromRow,
    {
        fetch(self, params)
    }

    fn fetch_all<P, R>(&self, params: &P) -> Result<Vec<R>, Error>
    where
        P: SqlQuery<R> + SqlParams,
        R: FromRow,
    {
        fetch_all(self, params)
    }

    fn select<T: SqlQuery<T> + SqlParams, F, R>(&self, entity: &T, to_model: F) -> Result<R, Error>
    where
        F: Fn(&Row) -> Result<R, Error>,
    {
        let sql = T::query();
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| *p as &dyn ToSql).collect();
        self.query_row(&sql, param_refs.as_slice(), to_model)
    }

    fn select_all<T: SqlQuery<T> + SqlParams, F, R>(
        &self,
        entity: &T,
        to_model: F,
    ) -> Result<Vec<R>, Error>
    where
        F: Fn(&Row) -> Result<R, Error>,
    {
        let sql = T::query();
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| *p as &dyn ToSql).collect();
        let mut stmt = self.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), to_model)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
}

/// # insert
///
/// Inserts a new record into the SQLite database.
///
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Data object to be inserted (must implement SqlCommand and SqlParams traits)
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
pub fn insert<T: SqlCommand + SqlParams, P: for<'a> FromSql + Send + Sync>(
    conn: &rusqlite::Connection,
    entity: T,
) -> Result<P, rusqlite::Error> {
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| *p as &dyn ToSql).collect();

    // Check if the SQL contains RETURNING clause
    if sql.to_uppercase().contains("RETURNING") {
        // Use query_row for RETURNING statements
        conn.query_row(&sql, param_refs.as_slice(), |row| {
            P::column_result(row.get_ref(0)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Integer,
                    Box::new(e),
                )
            })
        })
    } else {
        // Use execute for regular INSERT statements
        conn.execute(&sql, param_refs.as_slice())?;

        // Get the last inserted ID and use FromSql to convert it
        let last_id = conn.last_insert_rowid();
        P::column_result(rusqlite::types::ValueRef::Integer(last_id)).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Integer,
                Box::new(e),
            )
        })
    }
}

/// # update
///
/// Updates a record in the database.
///
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: The entity to update (must implement SqlCommand and UpdateParams traits)
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
///     // Create the update parameters (update user with id 1)
///     let update_query = UpdateUser {
///         id: 1,
///         name: String::from("John Updated"),
///         email: String::from("john.updated@example.com"),
///         state: 1,
///     };
///
///     // Execute update
///     let update_result = update(&conn, update_query)?;
///     println!("Update result: {:?}", update_result);
///     Ok(())
/// }
/// ```
pub fn update<T: SqlCommand + UpdateParams>(
    conn: &rusqlite::Connection,
    entity: T,
) -> Result<usize, Error> {
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| *p as &dyn ToSql).collect();
    let affected_rows = conn.execute(&sql, param_refs.as_slice())?;
    Ok(affected_rows)
}

/// # delete
///
/// Deletes a record from the database.
///
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Data object containing deletion parameters (must implement SqlCommand and SqlParams traits)
///
/// ## Return Value
/// - `Result<usize, Error>`: On success, returns the number of deleted records; on failure, returns Error
///
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
///
/// ```rust,no_run
/// #[derive(Deletable, SqlParams)]   // Required macros
/// #[table("table_name")]            // Table name to delete from
/// #[where_clause("id = ?")]         // Delete condition
/// pub struct MyEntity {
///     pub id: i64,                  // Field used in the condition
/// }
/// ```
///
/// - `Deletable`: Automatically generates SQL DELETE statements
/// - `SqlParams`: Automatically generates SQL parameters
/// - `#[table("table_name")]`: Specifies the table name for the deletion
/// - `#[where_clause("id = ?")]`: Specifies the delete condition (`?` will be replaced with parameter value)
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
pub fn delete<T: SqlCommand + SqlParams>(
    conn: &rusqlite::Connection,
    entity: T,
) -> Result<usize, Error> {
    let sql = T::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| *p as &dyn ToSql).collect();
    let affected_rows = conn.execute(&sql, param_refs.as_slice())?;
    Ok(affected_rows)
}

/// # fetch
///
/// Retrieves a single record from the database based on a specific condition.
///
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Query parameter object (must implement SqlQuery, FromRow, and SqlParams traits)
///
/// ## Return Value
/// - `Result<T, Error>`: On success, returns the queried record; on failure, returns Error
///
/// ## Struct Definition
/// Structs used with this function should be annotated with the following derive macros:
///
/// ```rust,no_run
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
/// use parsql_sqlite::fetch;
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
///     let user = fetch(&conn, &get_query)?;
///     println!("User: {:?}", user);
///     Ok(())
/// }
/// ```
pub fn fetch<P, R>(conn: &rusqlite::Connection, params: &P) -> Result<R, Error>
where
    P: SqlQuery<R> + SqlParams,
    R: FromRow,
{
    let sql = P::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
    }

    let query_params = params.params();
    let param_refs: Vec<&dyn ToSql> = query_params.iter().map(|p| *p as &dyn ToSql).collect();
    conn.query_row(&sql, param_refs.as_slice(), |row| R::from_row(row))
}

/// # fetch_all
///
/// Retrieves multiple records from the database based on a specific condition.
///
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Query parameter object (must implement SqlQuery, FromRow, and SqlParams traits)
///
/// ## Return Value
/// - `Result<Vec<T>, Error>`: On success, returns a vector of records; on failure, returns Error
///
/// ## Example Usage
///
/// ```rust,no_run
/// use rusqlite::{Connection, Result};
/// use parsql_macros::{Queryable, FromRow, SqlParams};
/// use parsql_sqlite::fetch_all;
///
/// fn main() -> Result<()> {
///     // Create database connection
///     let conn = Connection::open("test.db")?;
///     conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT, active INTEGER)", [])?;
///     conn.execute("INSERT INTO users (id, name, email, active) VALUES (1, 'John', 'john@example.com', 1)", [])?;
///     conn.execute("INSERT INTO users (id, name, email, active) VALUES (2, 'Jane', 'jane@example.com', 1)", [])?;
///
///     // Define a query
///     #[derive(Queryable, FromRow, SqlParams)]
///     #[table("users")]
///     #[where_clause("active = ?")]
///     pub struct GetActiveUsers {
///         pub id: i64,
///         pub name: String,
///         pub email: String,
///         pub active: i32,
///     }
///
///     // Create query parameters (get all active users)
///     let query = GetActiveUsers {
///         id: 0,
///         name: String::new(),
///         email: String::new(),
///         active: 1,
///     };
///
///     // Execute query
///     let users = fetch_all(&conn, &query)?;
///     println!("Active users: {:?}", users);
///     Ok(())
/// }
/// ```
pub fn fetch_all<P, R>(conn: &rusqlite::Connection, params: &P) -> Result<Vec<R>, Error>
where
    P: SqlQuery<R> + SqlParams,
    R: FromRow,
{
    let sql = P::query();
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
    }

    let query_params = params.params();
    let param_refs: Vec<&dyn ToSql> = query_params.iter().map(|p| *p as &dyn ToSql).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| R::from_row(row))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

/// # get
///
/// Retrieves a single record from the database based on a specific condition.
///
/// # Deprecated
/// This function has been renamed to `fetch`. Please use `fetch` instead.
///
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Query parameter object (must implement SqlQuery, FromRow, and SqlParams traits)
///
/// ## Return Value
/// - `Result<T, Error>`: On success, returns the queried record; on failure, returns Error
#[deprecated(
    since = "0.3.7",
    note = "Renamed to `fetch`. Please use `fetch` function instead."
)]
pub fn get<T: SqlQuery<T> + SqlParams, R>(
    conn: &rusqlite::Connection,
    entity: &T,
) -> Result<R, Error>
where
    T: SqlQuery<R> + SqlParams,
    R: FromRow,
{
    fetch(conn, entity)
}

/// # get_all
///
/// Retrieves multiple records from the database based on a specific condition.
///
/// # Deprecated
/// This function has been renamed to `fetch_all`. Please use `fetch_all` instead.
///
/// ## Parameters
/// - `conn`: SQLite database connection
/// - `entity`: Query parameter object (must implement SqlQuery, FromRow, and SqlParams traits)
///
/// ## Return Value
/// - `Result<Vec<T>, Error>`: On success, returns a vector of records; on failure, returns Error
#[deprecated(
    since = "0.3.7",
    note = "Renamed to `fetch_all`. Please use `fetch_all` function instead."
)]
pub fn get_all<T: SqlQuery<T> + SqlParams, R>(
    conn: &rusqlite::Connection,
    entity: &T,
) -> Result<Vec<R>, Error>
where
    T: SqlQuery<R> + SqlParams,
    R: FromRow,
{
    fetch_all(conn, entity)
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
///     let user = select(&conn, &get_query, to_user)?;
///     println!("User: {:?}", user);
///     Ok(())
/// }
/// ```
pub fn select<T: SqlQuery<T> + SqlParams, F, R>(
    conn: &rusqlite::Connection,
    entity: &T,
    to_model: F,
) -> Result<R, Error>
where
    F: Fn(&Row) -> Result<R, Error>,
{
    conn.select(entity, to_model)
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
///     let users = select_all(&conn, &get_query, to_user)?;
///     println!("Active users: {:?}", users);
///     Ok(())
/// }
/// ```
pub fn select_all<T: SqlQuery<T> + SqlParams, F, R>(
    conn: &rusqlite::Connection,
    entity: &T,
    to_model: F,
) -> Result<Vec<R>, Error>
where
    F: Fn(&Row) -> Result<R, Error>,
{
    conn.select_all(entity, to_model)
}
