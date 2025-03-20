use rusqlite::{Error, Row, ToSql};
use crate::{SqlQuery, SqlParams, UpdateParams, FromRow};

/// CrudOps trait defines the CRUD (Create, Read, Update, Delete) operations
/// that can be performed on a SQLite database.
///
/// This trait is implemented for the `rusqlite::Connection` struct, allowing
/// CRUD operations to be called as extension methods on a connection.
///
/// # Example
///
/// ```rust,no_run
/// use rusqlite::{Connection, Result};
/// use parsql::sqlite::CrudOps;
/// use parsql::sqlite::macros::{Insertable, SqlParams, Queryable, FromRow};
///
/// #[derive(Insertable, SqlParams)]
/// #[table("users")]
/// struct InsertUser {
///     name: String,
///     email: String,
/// }
///
/// #[derive(Queryable, FromRow, SqlParams)]
/// #[table("users")]
/// #[where_clause("id = ?")]
/// struct GetUser {
///     id: i64,
///     name: String,
///     email: String,
/// }
///
/// fn main() -> Result<()> {
///     let conn = Connection::open("test.db")?;
///    
///     // Extension method for insert
///     let insert_user = InsertUser {
///         name: "John".to_string(),
///         email: "john@example.com".to_string(),
///     };
///     let rows_affected = conn.insert(insert_user)?;
///    
///     // Extension method for get
///     let get_user = GetUser {
///         id: 1,
///         name: String::new(),
///         email: String::new(),
///     };
///     let user = conn.get(&get_user)?;
///    
///     println!("User: {:?}", user);
///     Ok(())
/// }
/// ```
pub trait CrudOps {
    /// Inserts a new record into the SQLite database.
    /// 
    /// # Arguments
    /// * `entity` - Data object to be inserted (must implement SqlQuery and SqlParams traits)
    /// 
    /// # Returns
    /// * `Result<usize, Error>` - On success, returns the number of inserted records; on failure, returns Error
    fn insert<T: SqlQuery + SqlParams>(&self, entity: T) -> Result<usize, Error>;

    /// Updates records in the SQLite database.
    /// 
    /// # Arguments
    /// * `entity` - Data object containing the update information (must implement SqlQuery and UpdateParams traits)
    /// 
    /// # Returns
    /// * `Result<usize, Error>` - On success, returns the number of updated records; on failure, returns Error
    fn update<T: SqlQuery + UpdateParams>(&self, entity: T) -> Result<usize, Error>;

    /// Deletes records from the SQLite database.
    /// 
    /// # Arguments
    /// * `entity` - Data object containing delete conditions (must implement SqlQuery and SqlParams traits)
    /// 
    /// # Returns
    /// * `Result<usize, Error>` - On success, returns the number of deleted records; on failure, returns Error
    fn delete<T: SqlQuery + SqlParams>(&self, entity: T) -> Result<usize, Error>;

    /// Retrieves a single record from the SQLite database.
    /// 
    /// # Arguments
    /// * `entity` - Data object containing query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
    /// 
    /// # Returns
    /// * `Result<T, Error>` - On success, returns the retrieved record; on failure, returns Error
    fn fetch<T: SqlQuery + FromRow + SqlParams>(&self, entity: &T) -> Result<T, Error>;

    /// Retrieves multiple records from the SQLite database.
    /// 
    /// # Arguments
    /// * `entity` - Data object containing query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
    /// 
    /// # Returns
    /// * `Result<Vec<T>, Error>` - On success, returns a vector of records; on failure, returns Error
    fn fetch_all<T: SqlQuery + FromRow + SqlParams>(&self, entity: &T) -> Result<Vec<T>, Error>;

    /// Executes a custom query and transforms the result using the provided function.
    /// 
    /// # Arguments
    /// * `entity` - Data object containing query parameters (must implement SqlQuery and SqlParams traits)
    /// * `to_model` - Function to transform the database row into the desired type
    /// 
    /// # Returns
    /// * `Result<R, Error>` - On success, returns the transformed result; on failure, returns Error
    fn select<T: SqlQuery + SqlParams, F, R>(&self, entity: &T, to_model: F) -> Result<R, Error>
    where
        F: Fn(&Row) -> Result<R, Error>;

    /// Executes a custom query and transforms all results using the provided function.
    /// 
    /// # Arguments
    /// * `entity` - Data object containing query parameters (must implement SqlQuery and SqlParams traits)
    /// * `to_model` - Function to transform database rows into the desired type
    /// 
    /// # Returns
    /// * `Result<Vec<R>, Error>` - On success, returns a vector of transformed results; on failure, returns Error
    fn select_all<T: SqlQuery + SqlParams, F, R>(&self, entity: &T, to_model: F) -> Result<Vec<R>, Error>
    where
        F: Fn(&Row) -> Result<R, Error>;
}

// CrudOps trait implementasyonu rusqlite::Connection için
impl CrudOps for rusqlite::Connection {
    fn insert<T: SqlQuery + SqlParams>(&self, entity: T) -> Result<usize, Error> {
        let sql = T::query();
        
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| *p as &dyn ToSql).collect();
        
        self.execute(&sql, param_refs.as_slice())
    }

    fn update<T: SqlQuery + UpdateParams>(&self, entity: T) -> Result<usize, Error> {
        let sql = T::query();
        
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| *p as &dyn ToSql).collect();
        
        self.execute(&sql, param_refs.as_slice())
    }

    fn delete<T: SqlQuery + SqlParams>(&self, entity: T) -> Result<usize, Error> {
        let sql = T::query();
        
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| *p as &dyn ToSql).collect();
        
        self.execute(&sql, param_refs.as_slice())
    }

    fn fetch<T: SqlQuery + FromRow + SqlParams>(&self, entity: &T) -> Result<T, Error> {
        let sql = T::query();
        
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| *p as &dyn ToSql).collect();
        
        let mut stmt = self.prepare(&sql)?;
        let mut rows = stmt.query(param_refs.as_slice())?;
        
        if let Some(row) = rows.next()? {
            let result = T::from_row(row)?;
            Ok(result)
        } else {
            Err(Error::QueryReturnedNoRows)
        }
    }

    fn fetch_all<T: SqlQuery + FromRow + SqlParams>(&self, entity: &T) -> Result<Vec<T>, Error> {
        let sql = T::query();
        
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-SQLITE] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| *p as &dyn ToSql).collect();
        
        let mut stmt = self.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), |row| T::from_row(row))?;
        
        let mut results = Vec::new();
        for row_result in rows {
            results.push(row_result?);
        }
        
        Ok(results)
    }

    fn select<T: SqlQuery + SqlParams, F, R>(&self, entity: &T, to_model: F) -> Result<R, Error>
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
        stmt.query_row(param_refs.as_slice(), to_model)
    }

    fn select_all<T: SqlQuery + SqlParams, F, R>(&self, entity: &T, to_model: F) -> Result<Vec<R>, Error>
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
    conn.insert(entity)
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
    conn.update(entity)
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
    conn.delete(entity)
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
pub fn fetch<T: SqlQuery + FromRow + SqlParams>(
    conn: &rusqlite::Connection,
    entity: &T,
) -> Result<T, Error> {
    conn.fetch(entity)
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
pub fn fetch_all<T: SqlQuery + FromRow + SqlParams>(
    conn: &rusqlite::Connection,
    entity: &T,
) -> Result<Vec<T>, Error> {
    conn.fetch_all(entity)
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
pub fn select<T: SqlQuery + SqlParams, F, R>(
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
///     let users = select_all(&mut conn, get_query, to_user)?;
///     println!("Active users: {:?}", users);
///     Ok(())
/// }
/// ```
pub fn select_all<T: SqlQuery + SqlParams, F, R>(
    conn: &rusqlite::Connection,
    entity: &T,
    to_model: F,
) -> Result<Vec<R>, Error>
where
    F: Fn(&Row) -> Result<R, Error>,
{
    conn.select_all(entity, to_model)
}
