use postgres::{
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

/// A trait for extending PostgreSQL client with CRUD operations.
///
/// This trait provides extension methods for tokio_postgres::Client to perform
/// common database CRUD operations in a more ergonomic way.
#[async_trait::async_trait]
pub trait CrudOps {
    /// Inserts a new record into the database.
    ///
    /// # Arguments
    /// * `entity` - Data object to be inserted (must implement SqlCommand and SqlParams traits)
    ///
    /// # Return Value
    /// * `Result<P, Error>` - On success, returns the inserted ID or return value; on failure, returns Error
    ///
    /// # Example
    /// ```rust,no_run
    /// # use tokio_postgres::{NoTls, Client};
    /// # use parsql::tokio_postgres::CrudOps;
    /// # use parsql::macros::{Insertable, SqlParams};
    /// #
    /// #[derive(Insertable, SqlParams)]
    /// #[table("users")]
    /// struct InsertUser {
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let (client, connection) = tokio_postgres::connect("", NoTls).await?;
    /// # tokio::spawn(async move { connection.await; });
    /// let user = InsertUser {
    ///     name: "John".to_string(),
    ///     email: "john@example.com".to_string(),
    /// };
    ///
    /// let id = client.insert(user).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn insert<T, P: for<'a> FromSql<'a> + Send + Sync>(&self, entity: T) -> Result<P, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync + 'static;

    /// Updates an existing record in the database.
    ///
    /// # Arguments
    /// * `entity` - Data object containing the update information (must implement SqlCommand and UpdateParams traits)
    ///
    /// # Return Value
    /// * `Result<bool, Error>` - On success, returns true if at least one record was updated; on failure, returns Error
    ///
    /// # Example
    /// ```rust,no_run
    /// # use tokio_postgres::{NoTls, Client};
    /// # use parsql::tokio_postgres::CrudOps;
    /// # use parsql::macros::{Updateable, UpdateParams};
    /// #
    /// #[derive(Updateable, UpdateParams)]
    /// #[table("users")]
    /// #[update("name, email")]
    /// #[where_clause("id = $")]
    /// struct UpdateUser {
    ///     id: i64,
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let (client, connection) = tokio_postgres::connect("", NoTls).await?;
    /// # tokio::spawn(async move { connection.await; });
    /// let user = UpdateUser {
    ///     id: 1,
    ///     name: "John Smith".to_string(),
    ///     email: "john.smith@example.com".to_string(),
    /// };
    ///
    /// let updated = client.update(user).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn update<T>(&self, entity: T) -> Result<bool, Error>
    where
        T: SqlCommand + UpdateParams + Send + Sync + 'static;

    /// Deletes a record from the database.
    ///
    /// # Arguments
    /// * `entity` - Data object containing delete conditions (must implement SqlCommand and SqlParams traits)
    ///
    /// # Return Value
    /// * `Result<u64, Error>` - On success, returns the number of deleted records; on failure, returns Error
    ///
    /// # Example
    /// ```rust,no_run
    /// # use tokio_postgres::{NoTls, Client};
    /// # use parsql::tokio_postgres::CrudOps;
    /// # use parsql::macros::{Deletable, SqlParams};
    /// #
    /// #[derive(Deletable, SqlParams)]
    /// #[table("users")]
    /// #[where_clause("id = $")]
    /// struct DeleteUser {
    ///     id: i64,
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let (client, connection) = tokio_postgres::connect("", NoTls).await?;
    /// # tokio::spawn(async move { connection.await; });
    /// let user = DeleteUser { id: 1 };
    ///
    /// let deleted = client.delete(user).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn delete<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync + 'static;

    /// Retrieves a single record from the database and converts it to a struct.
    ///
    /// # Arguments
    /// * `params` - Data object containing query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
    ///
    /// # Return Value
    /// * `Result<T, Error>` - On success, returns the retrieved record as a struct; on failure, returns Error
    ///
    /// # Example
    /// ```rust,no_run
    /// # use tokio_postgres::{NoTls, Client};
    /// # use parsql::tokio_postgres::CrudOps;
    /// # use parsql::macros::{Queryable, FromRow, SqlParams};
    /// #
    /// #[derive(Queryable, FromRow, SqlParams, Debug)]
    /// #[table("users")]
    /// #[where_clause("id = $")]
    /// struct GetUser {
    ///     id: i64,
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let (client, connection) = tokio_postgres::connect("", NoTls).await?;
    /// # tokio::spawn(async move { connection.await; });
    /// let query = GetUser {
    ///     id: 1,
    ///     name: Default::default(),
    ///     email: Default::default(),
    /// };
    ///
    /// let user = client.fetch(query).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn fetch<P, R>(&self, params: P) -> Result<R, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync + 'static,
        R: FromRow + Send + Sync + 'static;

    /// Retrieves multiple records from the database and converts them to a vec of structs.
    ///
    /// # Arguments
    /// * `params` - Data object containing query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
    ///
    /// # Return Value
    /// * `Result<Vec<T>, Error>` - On success, returns a vector of retrieved records; on failure, returns Error
    ///
    /// # Example
    /// ```rust,no_run
    /// # use tokio_postgres::{NoTls, Client};
    /// # use parsql::tokio_postgres::CrudOps;
    /// # use parsql::macros::{Queryable, FromRow, SqlParams};
    /// #
    /// #[derive(Queryable, FromRow, SqlParams, Debug)]
    /// #[table("users")]
    /// #[where_clause("state = $")]
    /// struct GetActiveUsers {
    ///     id: i64,
    ///     name: String,
    ///     email: String,
    ///     state: i16,
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let (client, connection) = tokio_postgres::connect("", NoTls).await?;
    /// # tokio::spawn(async move { connection.await; });
    /// let query = GetActiveUsers {
    ///     id: 0,
    ///     name: Default::default(),
    ///     email: Default::default(),
    ///     state: 1, // active users
    /// };
    ///
    /// let users = client.fetch_all(query).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn fetch_all<P, R>(&self, params: P) -> Result<Vec<R>, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync + 'static,
        R: FromRow + Send + Sync + 'static;

    /// Executes a custom SELECT query and converts the results using the provided function.
    ///
    /// # Arguments
    /// * `entity` - Data object containing query parameters (must implement SqlQuery and SqlParams traits)
    /// * `to_model` - Function to convert a row to the desired type
    ///
    /// # Return Value
    /// * `Result<R, Error>` - On success, returns the converted record; on failure, returns Error
    ///
    /// # Example
    /// ```rust,no_run
    /// # use tokio_postgres::{NoTls, Client, Row};
    /// # use parsql::tokio_postgres::CrudOps;
    /// # use parsql::macros::{Queryable, SqlParams};
    /// #
    /// #[derive(Queryable, SqlParams)]
    /// #[table("users")]
    /// #[select("SELECT u.*, p.role FROM users u JOIN profiles p ON u.id = p.user_id")]
    /// #[where_clause("u.state = $")]
    /// struct UserQuery {
    ///     state: i16,
    /// }
    ///
    /// struct UserWithRole {
    ///     id: i64,
    ///     name: String,
    ///     role: String,
    /// }
    ///
    /// fn convert_row(row: &Row) -> Result<UserWithRole, tokio_postgres::Error> {
    ///     Ok(UserWithRole {
    ///         id: row.try_get("id")?,
    ///         name: row.try_get("name")?,
    ///         role: row.try_get("role")?,
    ///     })
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let (client, connection) = tokio_postgres::connect("", NoTls).await?;
    /// # tokio::spawn(async move { connection.await; });
    /// let query = UserQuery { state: 1 };
    ///
    /// let user = client.select(query, convert_row).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn select<T, F, R>(&self, entity: T, to_model: F) -> Result<R, Error>
    where
        T: SqlQuery<T> + SqlParams + Send + Sync + 'static,
        F: Fn(&Row) -> Result<R, Error> + Send + Sync + 'static,
        R: Send + 'static;

    /// Executes a custom SELECT query and converts all the results using the provided function.
    ///
    /// # Arguments
    /// * `entity` - Data object containing query parameters (must implement SqlQuery and SqlParams traits)
    /// * `to_model` - Function to convert a row to the desired type
    ///
    /// # Return Value
    /// * `Result<Vec<R>, Error>` - On success, returns a vector of converted records; on failure, returns Error
    ///
    /// # Example
    /// ```rust,no_run
    /// # use tokio_postgres::{NoTls, Client, Row};
    /// # use parsql::tokio_postgres::CrudOps;
    /// # use parsql::macros::{Queryable, SqlParams};
    /// #
    /// #[derive(Queryable, SqlParams)]
    /// #[table("users")]
    /// #[select("SELECT u.*, p.role FROM users u JOIN profiles p ON u.id = p.user_id")]
    /// #[where_clause("u.state = $")]
    /// struct UserQuery {
    ///     state: i16,
    /// }
    ///
    /// struct UserWithRole {
    ///     id: i64,
    ///     name: String,
    ///     role: String,
    /// }
    ///
    /// fn convert_row(row: &Row) -> UserWithRole {
    ///     UserWithRole {
    ///         id: row.get("id"),
    ///         name: row.get("name"),
    ///         role: row.get("role"),
    ///     }
    /// }
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let (client, connection) = tokio_postgres::connect("", NoTls).await?;
    /// # tokio::spawn(async move { connection.await; });
    /// let query = UserQuery { state: 1 };
    ///
    /// let users = client.select_all(query, convert_row).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn select_all<T, F, R>(&self, entity: T, to_model: F) -> Result<Vec<R>, Error>
    where
        T: SqlQuery<T> + SqlParams + Send + Sync + 'static,
        F: Fn(&Row) -> R + Send + Sync + 'static,
        R: Send + 'static;

    #[deprecated(
        since = "0.2.0",
        note = "Renamed to `fetch`. Please use `fetch` function instead."
    )]
    async fn get<T>(&self, params: T) -> Result<T, Error>
    where
        T: SqlQuery<T> + FromRow + SqlParams + Send + Sync + 'static,
    {
        self.fetch(params).await
    }

    #[deprecated(
        since = "0.2.0",
        note = "Renamed to `fetch_all`. Please use `fetch_all` function instead."
    )]
    async fn get_all<T>(&self, params: T) -> Result<Vec<T>, Error>
    where
        T: SqlQuery<T> + FromRow + SqlParams + Send + Sync + 'static,
    {
        self.fetch_all(params).await
    }
}
