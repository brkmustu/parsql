use rusqlite::{
    types::{FromSql, ToSql},
    Error, Row,
};

/// Trait for generating SQL queries (for SELECT operations).
/// This trait is implemented by the derive macro `Queryable`.
pub trait SqlQuery<R> {
    /// Returns the SQL query string.
    fn query() -> String;
}

/// Trait for generating SQL commands (for INSERT/UPDATE/DELETE operations).
/// This trait is implemented by the derive macros `Insertable`, `Updateable`, and `Deletable`.
pub trait SqlCommand {
    /// Returns the SQL command string.
    fn query() -> String;
}

/// Trait for providing SQL parameters.
/// This trait is implemented by the derive macro `SqlParams`.
pub trait SqlParams {
    /// Returns a vector of references to SQL parameters.
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

/// Trait for providing UPDATE parameters.
/// This trait is implemented by the derive macro `UpdateParams`.
pub trait UpdateParams {
    /// Returns a vector of references to SQL parameters for UPDATE operations.
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

/// Trait for converting database rows to Rust structs.
/// This trait is implemented by the derive macro `FromRow`.
pub trait FromRow {
    /// Converts a database row to a Rust struct.
    ///
    /// # Arguments
    /// * `row` - A reference to a database row
    ///
    /// # Returns
    /// * `Result<Self, Error>` - The converted struct or an error
    fn from_row(row: &Row) -> Result<Self, Error>
    where
        Self: Sized;
}

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
    /// * `entity` - Data object to be inserted (must implement SqlCommand and SqlParams traits)
    ///
    /// # Returns
    /// * `Result<P, Error>` - On success, returns the inserted ID or return value; on failure, returns Error
    fn insert<T: SqlCommand + SqlParams, P: for<'a> FromSql + Send + Sync>(
        &self,
        entity: T,
    ) -> Result<P, Error>;

    /// Updates records in the SQLite database.
    ///
    /// # Arguments
    /// * `entity` - Data object containing the update information (must implement SqlCommand and UpdateParams traits)
    ///
    /// # Returns
    /// * `Result<usize, Error>` - On success, returns the number of updated records; on failure, returns Error
    fn update<T: SqlCommand + UpdateParams>(&self, entity: T) -> Result<usize, Error>;

    /// Deletes records from the SQLite database.
    ///
    /// # Arguments
    /// * `entity` - Data object containing delete conditions (must implement SqlCommand and SqlParams traits)
    ///
    /// # Returns
    /// * `Result<usize, Error>` - On success, returns the number of deleted records; on failure, returns Error
    fn delete<T: SqlCommand + SqlParams>(&self, entity: T) -> Result<usize, Error>;

    /// Retrieves a single record from the SQLite database.
    ///
    /// # Arguments
    /// * `params` - Query parameters (must implement SqlQuery and SqlParams traits)
    ///
    /// # Returns
    /// * `Result<R, Error>` - On success, returns the retrieved record; on failure, returns Error
    fn fetch<P, R>(&self, params: &P) -> Result<R, Error>
    where
        P: SqlQuery<R> + SqlParams,
        R: FromRow;

    /// Retrieves multiple records from the SQLite database.
    ///
    /// # Arguments
    /// * `params` - Query parameters (must implement SqlQuery and SqlParams traits)
    ///
    /// # Returns
    /// * `Result<Vec<R>, Error>` - On success, returns a vector of records; on failure, returns Error
    fn fetch_all<P, R>(&self, params: &P) -> Result<Vec<R>, Error>
    where
        P: SqlQuery<R> + SqlParams,
        R: FromRow;

    /// Retrieves a single record from the SQLite database.
    ///
    /// # Deprecated
    /// This function has been renamed to `fetch`. Please use `fetch` instead.
    ///
    /// # Arguments
    /// * `entity` - Data object containing query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
    ///
    /// # Returns
    /// * `Result<T, Error>` - On success, returns the retrieved record; on failure, returns Error
    #[deprecated(
        since = "0.3.7",
        note = "Renamed to `fetch`. Please use `fetch` function instead."
    )]
    fn get<T: SqlQuery<T> + FromRow + SqlParams>(&self, entity: &T) -> Result<T, Error> {
        self.fetch(entity)
    }

    /// Retrieves multiple records from the SQLite database.
    ///
    /// # Deprecated
    /// This function has been renamed to `fetch_all`. Please use `fetch_all` instead.
    ///
    /// # Arguments
    /// * `entity` - Data object containing query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
    ///
    /// # Returns
    /// * `Result<Vec<T>, Error>` - A vector of retrieved records or an error
    #[deprecated(
        since = "0.3.7",
        note = "Renamed to `fetch_all`. Please use `fetch_all` function instead."
    )]
    fn get_all<T: SqlQuery<T> + FromRow + SqlParams>(&self, entity: &T) -> Result<Vec<T>, Error> {
        self.fetch_all(entity)
    }

    /// Executes a custom query and transforms the result using the provided function.
    ///
    /// # Arguments
    /// * `entity` - Data object containing query parameters (must implement SqlQuery and SqlParams traits)
    /// * `to_model` - Function to transform the database row into the desired type
    ///
    /// # Returns
    /// * `Result<R, Error>` - On success, returns the transformed result; on failure, returns Error
    fn select<T: SqlQuery<T> + SqlParams, F, R>(&self, entity: &T, to_model: F) -> Result<R, Error>
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
    fn select_all<T: SqlQuery<T> + SqlParams, F, R>(
        &self,
        entity: &T,
        to_model: F,
    ) -> Result<Vec<R>, Error>
    where
        F: Fn(&Row) -> Result<R, Error>;
}
