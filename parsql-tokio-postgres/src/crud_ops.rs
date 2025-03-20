use tokio_postgres::{Error, Row, Client, Transaction};
use std::sync::OnceLock;
use crate::{SqlQuery, SqlParams, UpdateParams, FromRow};

/// A trait for extending PostgreSQL client with CRUD operations.
///
/// This trait provides extension methods for tokio_postgres::Client to perform
/// common database CRUD operations in a more ergonomic way.
#[async_trait::async_trait]
pub trait CrudOps {
    /// Inserts a new record into the database.
    ///
    /// # Arguments
    /// * `entity` - Data object to be inserted (must implement SqlQuery and SqlParams traits)
    ///
    /// # Return Value
    /// * `Result<u64, Error>` - On success, returns the number of inserted records; on failure, returns Error
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
    async fn insert<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlQuery + SqlParams + Send + Sync + 'static;

    /// Updates an existing record in the database.
    ///
    /// # Arguments
    /// * `entity` - Data object containing the update information (must implement SqlQuery and UpdateParams traits)
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
        T: SqlQuery + UpdateParams + Send + Sync + 'static;

    /// Deletes a record from the database.
    ///
    /// # Arguments
    /// * `entity` - Data object containing delete conditions (must implement SqlQuery and SqlParams traits)
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
        T: SqlQuery + SqlParams + Send + Sync + 'static;

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
    async fn fetch<T>(&self, params: T) -> Result<T, Error>
    where
        T: SqlQuery + FromRow + SqlParams + Send + Sync + 'static;

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
    async fn fetch_all<T>(&self, params: T) -> Result<Vec<T>, Error>
    where
        T: SqlQuery + FromRow + SqlParams + Send + Sync + 'static;

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
        T: SqlQuery + SqlParams + Send + Sync + 'static,
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
        T: SqlQuery + SqlParams + Send + Sync + 'static,
        F: Fn(&Row) -> R + Send + Sync + 'static,
        R: Send + 'static;

    #[deprecated(
        since = "0.2.0",
        note = "Renamed to `fetch`. Please use `fetch` function instead."
    )]
    async fn get<T>(&self, params: T) -> Result<T, Error>
    where
        T: SqlQuery + FromRow + SqlParams + Send + Sync + 'static,
    {
        self.fetch(params).await
    }

    #[deprecated(
        since = "0.2.0",
        note = "Renamed to `fetch_all`. Please use `fetch_all` function instead."
    )]
    async fn get_all<T>(&self, params: T) -> Result<Vec<T>, Error>
    where
        T: SqlQuery + FromRow + SqlParams + Send + Sync + 'static,
    {
        self.fetch_all(params).await
    }
}

#[async_trait::async_trait]
impl CrudOps for Client {
    async fn insert<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlQuery + SqlParams + Send + Sync + 'static,
    {
        let sql = T::query();
        
        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled = *TRACE_ENABLED.get_or_init(|| {
            std::env::var("PARSQL_TRACE").unwrap_or_default() == "1"
        });
        
        if is_trace_enabled {
            println!("[PARSQL-TOKIO-POSTGRES] Execute SQL: {}", sql);
        }

        let params = entity.params();
        self.execute(&sql, &params).await
    }

    async fn update<T>(&self, entity: T) -> Result<bool, Error>
    where
        T: SqlQuery + UpdateParams + Send + Sync + 'static,
    {
        let sql = T::query();
        
        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled = *TRACE_ENABLED.get_or_init(|| {
            std::env::var("PARSQL_TRACE").unwrap_or_default() == "1"
        });
        
        if is_trace_enabled {
            println!("[PARSQL-TOKIO-POSTGRES] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let result = self.execute(&sql, &params).await?;
        Ok(result > 0)
    }

    async fn delete<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlQuery + SqlParams + Send + Sync + 'static,
    {
        let sql = T::query();
        
        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled = *TRACE_ENABLED.get_or_init(|| {
            std::env::var("PARSQL_TRACE").unwrap_or_default() == "1"
        });
        
        if is_trace_enabled {
            println!("[PARSQL-TOKIO-POSTGRES] Execute SQL: {}", sql);
        }

        let params = entity.params();
        self.execute(&sql, &params).await
    }

    async fn fetch<T>(&self, params: T) -> Result<T, Error>
    where
        T: SqlQuery + FromRow + SqlParams + Send + Sync + 'static,
    {
        let sql = T::query();
        
        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled = *TRACE_ENABLED.get_or_init(|| {
            std::env::var("PARSQL_TRACE").unwrap_or_default() == "1"
        });
        
        if is_trace_enabled {
            println!("[PARSQL-TOKIO-POSTGRES] Execute SQL: {}", sql);
        }

        let query_params = params.params();
        let row = self.query_one(&sql, &query_params).await?;
        T::from_row(&row)
    }

    async fn fetch_all<T>(&self, params: T) -> Result<Vec<T>, Error>
    where
        T: SqlQuery + FromRow + SqlParams + Send + Sync + 'static,
    {
        let sql = T::query();
        
        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled = *TRACE_ENABLED.get_or_init(|| {
            std::env::var("PARSQL_TRACE").unwrap_or_default() == "1"
        });
        
        if is_trace_enabled {
            println!("[PARSQL-TOKIO-POSTGRES] Execute SQL: {}", sql);
        }

        let query_params = params.params();
        let rows = self.query(&sql, &query_params).await?;
        
        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(T::from_row(&row)?);
        }
        
        Ok(results)
    }

    async fn select<T, F, R>(&self, entity: T, to_model: F) -> Result<R, Error>
    where
        T: SqlQuery + SqlParams + Send + Sync + 'static,
        F: Fn(&Row) -> Result<R, Error> + Send + Sync + 'static,
        R: Send + 'static,
    {
        let sql = T::query();
        
        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled = *TRACE_ENABLED.get_or_init(|| {
            std::env::var("PARSQL_TRACE").unwrap_or_default() == "1"
        });
        
        if is_trace_enabled {
            println!("[PARSQL-TOKIO-POSTGRES] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let row = self.query_one(&sql, &params).await?;
        to_model(&row)
    }

    async fn select_all<T, F, R>(&self, entity: T, to_model: F) -> Result<Vec<R>, Error>
    where
        T: SqlQuery + SqlParams + Send + Sync + 'static,
        F: Fn(&Row) -> R + Send + Sync + 'static,
        R: Send + 'static,
    {
        let sql = T::query();
        
        static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
        let is_trace_enabled = *TRACE_ENABLED.get_or_init(|| {
            std::env::var("PARSQL_TRACE").unwrap_or_default() == "1"
        });
        
        if is_trace_enabled {
            println!("[PARSQL-TOKIO-POSTGRES] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let rows = self.query(&sql, &params).await?;
        
        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(to_model(&row));
        }
        
        Ok(results)
    }
}

/// # insert
/// 
/// Inserts a new record into the database.
/// 
/// ## Parameters
/// - `client`: Database connection object
/// - `entity`: Data object to be inserted (must implement SqlQuery and SqlParams traits)
/// 
/// ## Return Value
/// - `Result<u64, Error>`: On success, returns the number of inserted records; on failure, returns Error
pub async fn insert<T>(
    client: &Client,
    entity: T,
) -> Result<u64, Error>
where
    T: SqlQuery + SqlParams + Send + Sync + 'static
{
    client.insert(entity).await
}

/// # update
/// 
/// Updates an existing record in the database.
/// 
/// ## Parameters
/// - `client`: Database connection object
/// - `entity`: Data object containing the update information (must implement SqlQuery and UpdateParams traits)
/// 
/// ## Return Value
/// - `Result<bool, Error>`: On success, returns true; on failure, returns Error
pub async fn update<T>(
    client: &Client,
    entity: T,
) -> Result<bool, Error>
where
    T: SqlQuery + UpdateParams + Send + Sync + 'static
{
    client.update(entity).await
}

/// # delete
/// 
/// Deletes a record from the database.
/// 
/// ## Parameters
/// - `client`: Database connection object
/// - `entity`: Data object containing delete conditions (must implement SqlQuery and SqlParams traits)
/// 
/// ## Return Value
/// - `Result<u64, Error>`: On success, returns the number of deleted records; on failure, returns Error
pub async fn delete<T>(
    client: &Client,
    entity: T,
) -> Result<u64, Error>
where
    T: SqlQuery + SqlParams + Send + Sync + 'static
{
    client.delete(entity).await
}

/// # fetch
/// 
/// Retrieves a single record from the database and converts it to a struct.
/// 
/// ## Parameters
/// - `client`: Database connection object
/// - `params`: Data object containing query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
/// 
/// ## Return Value
/// - `Result<T, Error>`: On success, returns the retrieved record as a struct; on failure, returns Error
pub async fn fetch<T>(
    client: &Client,
    params: T,
) -> Result<T, Error>
where
    T: SqlQuery + FromRow + SqlParams + Send + Sync + 'static
{
    client.fetch(params).await
}

/// # fetch_all
/// 
/// Retrieves multiple records from the database.
/// 
/// ## Parameters
/// - `client`: Database connection object
/// - `params`: Query parameter object (must implement SqlQuery, FromRow, and SqlParams traits)
/// 
/// ## Return Value
/// - `Result<Vec<T>, Error>`: On success, returns the list of found records; on failure, returns Error
pub async fn fetch_all<T>(
    client: &Client,
    params: T,
) -> Result<Vec<T>, Error>
where
    T: SqlQuery + FromRow + SqlParams + Send + Sync + 'static
{
    client.fetch_all(params).await
}

/// # select
/// 
/// Retrieves a single record from the database using a custom transformation function.
/// This is useful when you want to use a custom transformation function instead of the FromRow trait.
/// 
/// ## Parameters
/// - `client`: Database connection object
/// - `entity`: Query parameter object (must implement SqlQuery and SqlParams traits)
/// - `to_model`: Function to convert a Row object to the target object type
/// 
/// ## Return Value
/// - `Result<R, Error>`: On success, returns the transformed object; on failure, returns Error
pub async fn select<T, F, R>(
    client: &Client,
    entity: T,
    to_model: F,
) -> Result<R, Error>
where
    T: SqlQuery + SqlParams + Send + Sync + 'static,
    F: Fn(&Row) -> Result<R, Error> + Send + Sync + 'static,
    R: Send + 'static
{
    client.select(entity, to_model).await
}

/// # select_all
/// 
/// Retrieves multiple records from the database using a custom transformation function.
/// This is useful when you want to use a custom transformation function instead of the FromRow trait.
/// 
/// ## Parameters
/// - `client`: Database connection object
/// - `entity`: Query parameter object (must implement SqlQuery and SqlParams traits)
/// - `to_model`: Function to convert a Row object to the target object type
/// 
/// ## Return Value
/// - `Result<Vec<R>, Error>`: On success, returns the list of transformed objects; on failure, returns Error
pub async fn select_all<T, F, R>(
    client: &Client,
    entity: T,
    to_model: F,
) -> Result<Vec<R>, Error>
where
    T: SqlQuery + SqlParams + Send + Sync + 'static,
    F: Fn(&Row) -> R + Send + Sync + 'static,
    R: Send + 'static
{
    client.select_all(entity, to_model).await
}

// DEPRECATED FUNCTIONS - For backward compatibility

/// # get
/// 
/// Retrieves a single record from the database and converts it to a struct.
/// 
/// # Deprecated
/// This function has been renamed to `fetch`. Please use `fetch` instead.
/// 
/// # Arguments
/// * `client` - Database connection client
/// * `params` - Query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
/// 
/// # Return Value
/// * `Result<T, Error>` - On success, returns the retrieved record; on failure, returns Error
#[deprecated(
    since = "0.2.0",
    note = "Renamed to `fetch`. Please use `fetch` function instead."
)]
pub async fn get<T>(
    client: &Client,
    params: T,
) -> Result<T, Error>
where
    T: SqlQuery + FromRow + SqlParams + Send + Sync + 'static
{
    fetch(client, params).await
}

/// # get_all
/// 
/// Retrieves multiple records from the database.
/// 
/// # Deprecated
/// This function has been renamed to `fetch_all`. Please use `fetch_all` instead.
/// 
/// # Arguments
/// * `client` - Database connection client
/// * `params` - Query parameters (must implement SqlQuery, FromRow, and SqlParams traits)
/// 
/// # Return Value
/// * `Result<Vec<T>, Error>` - On success, returns the list of found records; on failure, returns Error
#[deprecated(
    since = "0.2.0",
    note = "Renamed to `fetch_all`. Please use `fetch_all` function instead."
)]
pub async fn get_all<T>(
    client: &Client,
    params: T,
) -> Result<Vec<T>, Error>
where
    T: SqlQuery + FromRow + SqlParams + Send + Sync + 'static
{
    fetch_all(client, params).await
}
