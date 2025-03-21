//! Transaction operations for SQLite
//!
//! This module provides functions for performing CRUD operations within a transaction.

use rusqlite::{Connection, Transaction, Error, ToSql};
use crate::crud_ops::CrudOps;
use crate::{SqlParams, SqlQuery, UpdateParams, FromRow};

/// Implementation of CrudOps for Transaction
impl<'conn> CrudOps for Transaction<'conn> {
    /// Inserts a record into the database and returns the number of rows affected.
    /// This function is an extension to the Transaction struct and is available when the CrudOps trait is in scope.
    ///
    /// # Arguments
    /// * `entity` - A struct that implements Insertable and SqlParams traits
    ///
    /// # Returns
    /// * `Result<usize, Error>` - Number of affected rows or an error
    ///
    /// # Example
    /// ```rust,no_run
    /// use rusqlite::{Connection, Result};
    /// use parsql::sqlite::CrudOps;
    /// use parsql::sqlite::transactional;
    /// use parsql::macros::{Insertable, SqlParams};
    ///
    /// #[derive(Insertable, SqlParams)]
    /// #[table("users")]
    /// struct InsertUser {
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// fn main() -> Result<()> {
    ///     let conn = Connection::open("test.db")?;
    ///     let tx = transactional::begin(&conn)?;
    ///     
    ///     let user = InsertUser {
    ///         name: "John".to_string(),
    ///         email: "john@example.com".to_string(),
    ///     };
    ///     
    ///     let rows_affected = tx.insert(user)?;
    ///     
    ///     tx.commit()?;
    ///     Ok(())
    /// }
    /// ```
    fn insert<T: SqlQuery + SqlParams>(&self, entity: T) -> Result<usize, Error> {
        let sql = T::query();
        
        // Debug log the SQL query
        #[cfg(debug_assertions)]
        println!("[SQL] {}", sql);
        
        let params = entity.params();
        let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| *p as &dyn ToSql).collect();
        
        self.execute(&sql, param_refs.as_slice())
    }

    /// Updates a record in the database and returns the number of rows affected.
    /// This function is an extension to the Transaction struct and is available when the CrudOps trait is in scope.
    ///
    /// # Arguments
    /// * `entity` - A struct that implements Updateable and UpdateParams traits
    ///
    /// # Returns
    /// * `Result<usize, Error>` - Number of affected rows or an error
    ///
    /// # Example
    /// ```rust,no_run
    /// use rusqlite::{Connection, Result};
    /// use parsql::sqlite::CrudOps;
    /// use parsql::sqlite::transactional;
    /// use parsql::macros::{Updateable, UpdateParams};
    ///
    /// #[derive(Updateable, UpdateParams)]
    /// #[table("users")]
    /// #[update("name, email")]
    /// #[where_clause("id = ?")]
    /// struct UpdateUser {
    ///     id: i64,
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// fn main() -> Result<()> {
    ///     let conn = Connection::open("test.db")?;
    ///     let tx = transactional::begin(&conn)?;
    ///     
    ///     let user = UpdateUser {
    ///         id: 1,
    ///         name: "John Doe".to_string(),
    ///         email: "john.doe@example.com".to_string(),
    ///     };
    ///     
    ///     let rows_affected = tx.update(user)?;
    ///     
    ///     tx.commit()?;
    ///     Ok(())
    /// }
    /// ```
    fn update<T: SqlQuery + UpdateParams>(&self, entity: T) -> Result<usize, Error> {
        let sql = T::query();
        
        // Debug log the SQL query
        #[cfg(debug_assertions)]
        println!("[SQL] {}", sql);
        
        let params = entity.params();
        let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| *p as &dyn ToSql).collect();
        
        self.execute(&sql, param_refs.as_slice())
    }

    /// Deletes a record from the database and returns the number of rows affected.
    /// This function is an extension to the Transaction struct and is available when the CrudOps trait is in scope.
    ///
    /// # Arguments
    /// * `entity` - A struct that implements Deletable and SqlParams traits
    ///
    /// # Returns
    /// * `Result<usize, Error>` - Number of affected rows or an error
    ///
    /// # Example
    /// ```rust,no_run
    /// use rusqlite::{Connection, Result};
    /// use parsql::sqlite::CrudOps;
    /// use parsql::sqlite::transactional;
    /// use parsql::macros::{Deletable, SqlParams};
    ///
    /// #[derive(Deletable, SqlParams)]
    /// #[table("users")]
    /// #[where_clause("id = ?")]
    /// struct DeleteUser {
    ///     id: i64,
    /// }
    ///
    /// fn main() -> Result<()> {
    ///     let conn = Connection::open("test.db")?;
    ///     let tx = transactional::begin(&conn)?;
    ///     
    ///     let user = DeleteUser { id: 1 };
    ///     
    ///     let rows_affected = tx.delete(user)?;
    ///     
    ///     tx.commit()?;
    ///     Ok(())
    /// }
    /// ```
    fn delete<T: SqlQuery + SqlParams>(&self, entity: T) -> Result<usize, Error> {
        let sql = T::query();
        
        // Debug log the SQL query
        #[cfg(debug_assertions)]
        println!("[SQL] {}", sql);
        
        let params = entity.params();
        let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| *p as &dyn ToSql).collect();
        
        self.execute(&sql, param_refs.as_slice())
    }

    /// Retrieves a single record from the database and converts it to a struct.
    /// This function is an extension to the Transaction struct and is available when the CrudOps trait is in scope.
    ///
    /// # Arguments
    /// * `entity` - A struct that implements Queryable, SqlParams, and FromRow traits
    ///
    /// # Returns
    /// * `Result<T, Error>` - The retrieved record as a struct or an error
    ///
    /// # Example
    /// ```rust,no_run
    /// use rusqlite::{Connection, Result};
    /// use parsql::sqlite::CrudOps;
    /// use parsql::sqlite::transactional;
    /// use parsql::macros::{Queryable, SqlParams, FromRow};
    ///
    /// #[derive(Queryable, SqlParams, FromRow)]
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
    ///     let tx = transactional::begin(&conn)?;
    ///     
    ///     let param = GetUser {
    ///         id: 1,
    ///         name: String::new(),
    ///         email: String::new(),
    ///     };
    ///     
    ///     let user = tx.fetch(&param)?;
    ///     
    ///     tx.commit()?;
    ///     println!("Found user: {} - {}", user.name, user.email);
    ///     Ok(())
    /// }
    /// ```
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

    /// Retrieves multiple records from the database and converts them to a vector of structs.
    /// This function is an extension to the Transaction struct and is available when the CrudOps trait is in scope.
    ///
    /// # Arguments
    /// * `entity` - A struct that implements Queryable, SqlParams, and FromRow traits
    ///
    /// # Returns
    /// * `Result<Vec<T>, Error>` - A vector of retrieved records as structs or an error
    ///
    /// # Example
    /// ```rust,no_run
    /// use rusqlite::{Connection, Result};
    /// use parsql::sqlite::CrudOps;
    /// use parsql::sqlite::transactional;
    /// use parsql::macros::{Queryable, SqlParams, FromRow};
    ///
    /// #[derive(Queryable, SqlParams, FromRow)]
    /// #[table("users")]
    /// #[where_clause("email LIKE ?")]
    /// struct GetUsers {
    ///     id: i64,
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// fn main() -> Result<()> {
    ///     let conn = Connection::open("test.db")?;
    ///     let tx = transactional::begin(&conn)?;
    ///     
    ///     let param = GetUsers {
    ///         id: 0,
    ///         name: String::new(),
    ///         email: "%@example.com".to_string(),
    ///     };
    ///     
    ///     let users = tx.fetch_all(&param)?;
    ///     
    ///     tx.commit()?;
    ///     println!("Found {} users", users.len());
    ///     Ok(())
    /// }
    /// ```
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

    /// Executes a custom SELECT query and transforms the result using a provided function.
    /// This function is an extension to the Transaction struct and is available when the CrudOps trait is in scope.
    ///
    /// # Arguments
    /// * `entity` - Data object containing query parameters
    /// * `to_model` - Function to transform a row into a value
    ///
    /// # Returns
    /// * `Result<R, Error>` - The transformed value or an error
    ///
    /// # Example
    /// ```rust,no_run
    /// use rusqlite::{Connection, Result};
    /// use parsql::sqlite::CrudOps;
    /// use parsql::sqlite::transactional;
    /// use parsql::macros::{Queryable, SqlParams};
    ///
    /// #[derive(Queryable, SqlParams)]
    /// #[table("users")]
    /// #[where_clause("email LIKE ?")]
    /// struct CountUsers {
    ///     email: String,
    /// }
    ///
    /// fn main() -> Result<()> {
    ///     let conn = Connection::open("test.db")?;
    ///     let tx = transactional::begin(&conn)?;
    ///     
    ///     let param = CountUsers {
    ///         email: "%example.com".to_string(),
    ///     };
    ///     
    ///     let count: i64 = tx.select(&param, |row| row.get(0))?;
    ///     
    ///     tx.commit()?;
    ///     println!("Number of users: {}", count);
    ///     Ok(())
    /// }
    /// ```
    fn select<T: SqlQuery + SqlParams, F, R>(&self, entity: &T, to_model: F) -> Result<R, Error>
    where
        F: Fn(&rusqlite::Row) -> Result<R, Error>,
    {
        let sql = T::query();
        
        // Debug log the SQL query
        #[cfg(debug_assertions)]
        println!("[SQL] {}", sql);
        
        let params = entity.params();
        let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| *p as &dyn ToSql).collect();
        
        let mut stmt = self.prepare(&sql)?;
        stmt.query_row(param_refs.as_slice(), to_model)
    }

    /// Executes a custom SELECT query and transforms all results using a provided function.
    /// This function is an extension to the Transaction struct and is available when the CrudOps trait is in scope.
    ///
    /// # Arguments
    /// * `entity` - Data object containing query parameters
    /// * `to_model` - Function to transform rows into values
    ///
    /// # Returns
    /// * `Result<Vec<R>, Error>` - A vector of transformed values or an error
    ///
    /// # Example
    /// ```rust,no_run
    /// use rusqlite::{Connection, Result};
    /// use parsql::sqlite::CrudOps;
    /// use parsql::sqlite::transactional;
    /// use parsql::macros::{Queryable, SqlParams};
    ///
    /// #[derive(Queryable, SqlParams)]
    /// #[table("users")]
    /// #[where_clause("email LIKE ?")]
    /// struct GetUserNames {
    ///     email: String,
    /// }
    ///
    /// fn main() -> Result<()> {
    ///     let conn = Connection::open("test.db")?;
    ///     let tx = transactional::begin(&conn)?;
    ///     
    ///     let param = GetUserNames {
    ///         email: "%example.com".to_string(),
    ///     };
    ///     
    ///     let names: Vec<String> = tx.select_all(&param, |row| row.get(0))?;
    ///     
    ///     tx.commit()?;
    ///     for name in names {
    ///         println!("User name: {}", name);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn select_all<T: SqlQuery + SqlParams, F, R>(&self, entity: &T, to_model: F) -> Result<Vec<R>, Error>
    where
        F: Fn(&rusqlite::Row) -> Result<R, Error>,
    {
        let sql = T::query();
        
        // Debug log the SQL query
        #[cfg(debug_assertions)]
        println!("[SQL] {}", sql);
        
        let params = entity.params();
        let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| *p as &dyn ToSql).collect();
        
        let mut stmt = self.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), to_model)?;
        
        let mut results = Vec::new();
        for row_result in rows {
            results.push(row_result?);
        }
        
        Ok(results)
    }
}

/// Begins a new transaction.
///
/// # Arguments
/// * `conn` - SQLite connection
///
/// # Returns
/// * `Result<Transaction<'_>, Error>` - Transaction or an error
///
/// # Example
/// ```rust,no_run
/// use rusqlite::{Connection, Result};
/// use parsql::sqlite::transactional;
///
/// fn main() -> Result<()> {
///     let conn = Connection::open("test.db")?;
///     let tx = transactional::begin(&conn)?;
///     // Perform operations within the transaction
///     tx.commit()?;
///     Ok(())
/// }
/// ```
pub fn begin(conn: &Connection) -> Result<Transaction<'_>, Error> {
    conn.unchecked_transaction()
}

/// Inserts a record into the database within a transaction.
///
/// # Arguments
/// * `tx` - Transaction
/// * `entity` - A struct that implements Insertable and SqlParams traits
///
/// # Returns
/// * `Result<(Transaction<'_>, usize), Error>` - Transaction and number of affected rows or an error
///
/// # Example
/// ```rust,no_run
/// use rusqlite::{Connection, Result};
/// use parsql::sqlite::transactional;
/// use parsql::macros::{Insertable, SqlParams};
///
/// #[derive(Insertable, SqlParams)]
/// #[table("users")]
/// struct InsertUser {
///     name: String,
///     email: String,
/// }
///
/// fn main() -> Result<()> {
///     let conn = Connection::open("test.db")?;
///     let tx = transactional::begin(&conn)?;
///     
///     let user = InsertUser {
///         name: "John".to_string(),
///         email: "john@example.com".to_string(),
///     };
///     
///     let (tx, rows_affected) = transactional::tx_insert(tx, user)?;
///     
///     tx.commit()?;
///     Ok(())
/// }
/// ```
pub fn tx_insert<'a, T: SqlQuery + SqlParams>(
    tx: Transaction<'a>,
    entity: T,
) -> Result<(Transaction<'a>, usize), Error> {
    let result = tx.insert(entity)?;
    Ok((tx, result))
}

/// Updates a record in the database within a transaction.
///
/// # Arguments
/// * `tx` - Transaction
/// * `entity` - A struct that implements Updateable and UpdateParams traits
///
/// # Returns
/// * `Result<(Transaction<'_>, usize), Error>` - Transaction and number of affected rows or an error
///
/// # Example
/// ```rust,no_run
/// use rusqlite::{Connection, Result};
/// use parsql::sqlite::transactional;
/// use parsql::macros::{Updateable, UpdateParams};
///
/// #[derive(Updateable, UpdateParams)]
/// #[table("users")]
/// #[update("name, email")]
/// #[where_clause("id = ?")]
/// struct UpdateUser {
///     id: i64,
///     name: String,
///     email: String,
/// }
///
/// fn main() -> Result<()> {
///     let conn = Connection::open("test.db")?;
///     let tx = transactional::begin(&conn)?;
///     
///     let user = UpdateUser {
///         id: 1,
///         name: "John Doe".to_string(),
///         email: "john.doe@example.com".to_string(),
///     };
///     
///     let (tx, rows_affected) = transactional::tx_update(tx, user)?;
///     
///     tx.commit()?;
///     Ok(())
/// }
/// ```
pub fn tx_update<'a, T: SqlQuery + UpdateParams>(
    tx: Transaction<'a>,
    entity: T,
) -> Result<(Transaction<'a>, usize), Error> {
    let result = tx.update(entity)?;
    Ok((tx, result))
}

/// Deletes a record from the database within a transaction.
///
/// # Arguments
/// * `tx` - Transaction
/// * `entity` - A struct that implements Deletable and SqlParams traits
///
/// # Returns
/// * `Result<(Transaction<'_>, usize), Error>` - Transaction and number of affected rows or an error
///
/// # Example
/// ```rust,no_run
/// use rusqlite::{Connection, Result};
/// use parsql::sqlite::transactional;
/// use parsql::macros::{Deletable, SqlParams};
///
/// #[derive(Deletable, SqlParams)]
/// #[table("users")]
/// #[where_clause("id = ?")]
/// struct DeleteUser {
///     id: i64,
/// }
///
/// fn main() -> Result<()> {
///     let conn = Connection::open("test.db")?;
///     let tx = transactional::begin(&conn)?;
///     
///     let user = DeleteUser { id: 1 };
///     
///     let (tx, rows_affected) = transactional::tx_delete(tx, user)?;
///     
///     tx.commit()?;
///     Ok(())
/// }
/// ```
pub fn tx_delete<'a, T: SqlQuery + SqlParams>(
    tx: Transaction<'a>,
    entity: T,
) -> Result<(Transaction<'a>, usize), Error> {
    let result = tx.delete(entity)?;
    Ok((tx, result))
}

/// Fetches a single record from the database within a transaction.
///
/// # Arguments
/// * `tx` - Transaction
/// * `entity` - A struct that implements Queryable, SqlParams, and FromRow traits
///
/// # Returns
/// * `Result<(Transaction<'_>, T), Error>` - Transaction and the retrieved record or an error
///
/// # Example
/// ```rust,no_run
/// use rusqlite::{Connection, Result};
/// use parsql::sqlite::transactional;
/// use parsql::macros::{Queryable, SqlParams, FromRow};
///
/// #[derive(Queryable, SqlParams, FromRow)]
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
///     let tx = transactional::begin(&conn)?;
///     
///     let param = GetUser {
///         id: 1,
///         name: String::new(),
///         email: String::new(),
///     };
///     
///     let (tx, user) = transactional::tx_fetch(tx, &param)?;
///     
///     println!("Found user: {} - {}", user.name, user.email);
///     
///     tx.commit()?;
///     Ok(())
/// }
/// ```
pub fn tx_fetch<'a, T: SqlQuery + FromRow + SqlParams>(
    tx: Transaction<'a>,
    entity: &T,
) -> Result<(Transaction<'a>, T), Error> {
    let result = tx.fetch(entity)?;
    Ok((tx, result))
}

/// Fetches multiple records from the database within a transaction.
///
/// # Arguments
/// * `tx` - Transaction
/// * `entity` - A struct that implements Queryable, SqlParams, and FromRow traits
///
/// # Returns
/// * `Result<(Transaction<'_>, Vec<T>), Error>` - Transaction and a vector of retrieved records or an error
///
/// # Example
/// ```rust,no_run
/// use rusqlite::{Connection, Result};
/// use parsql::sqlite::transactional;
/// use parsql::macros::{Queryable, SqlParams, FromRow};
///
/// #[derive(Queryable, SqlParams, FromRow)]
/// #[table("users")]
/// #[where_clause("active = ?")]
/// struct GetActiveUsers {
///     id: i64,
///     name: String,
///     email: String,
///     active: i32,
/// }
///
/// fn main() -> Result<()> {
///     let conn = Connection::open("test.db")?;
///     let tx = transactional::begin(&conn)?;
///     
///     let param = GetActiveUsers {
///         id: 0,
///         name: String::new(),
///         email: String::new(),
///         active: 1,
///     };
///     
///     let (tx, users) = transactional::tx_fetch_all(tx, &param)?;
///     
///     println!("Found {} active users", users.len());
///     
///     tx.commit()?;
///     Ok(())
/// }
/// ```
pub fn tx_fetch_all<'a, T: SqlQuery + FromRow + SqlParams>(
    tx: Transaction<'a>,
    entity: &T,
) -> Result<(Transaction<'a>, Vec<T>), Error> {
    let results = tx.fetch_all(entity)?;
    Ok((tx, results))
}

/// Gets a single record from the database within a transaction.
/// 
/// # Deprecated
/// This function has been renamed to `tx_fetch`. Please use `tx_fetch` instead.
///
/// # Arguments
/// * `tx` - Transaction
/// * `entity` - A struct that implements Queryable, SqlParams, and FromRow traits
///
/// # Returns
/// * `Result<(Transaction<'_>, T), Error>` - Transaction and the retrieved record or an error
#[deprecated(
    since = "0.3.7",
    note = "Renamed to `tx_fetch`. Please use `tx_fetch` function instead."
)]
pub fn tx_get<'a, T: SqlQuery + FromRow + SqlParams>(
    tx: Transaction<'a>,
    entity: &T,
) -> Result<(Transaction<'a>, T), Error> {
    tx_fetch(tx, entity)
}

/// Gets multiple records from the database within a transaction.
/// 
/// # Deprecated
/// This function has been renamed to `tx_fetch_all`. Please use `tx_fetch_all` instead.
///
/// # Arguments
/// * `tx` - Transaction
/// * `entity` - A struct that implements Queryable, SqlParams, and FromRow traits
///
/// # Returns
/// * `Result<(Transaction<'_>, Vec<T>), Error>` - Transaction and a vector of retrieved records or an error
#[deprecated(
    since = "0.3.7",
    note = "Renamed to `tx_fetch_all`. Please use `tx_fetch_all` function instead."
)]
pub fn tx_get_all<'a, T: SqlQuery + FromRow + SqlParams>(
    tx: Transaction<'a>,
    entity: &T,
) -> Result<(Transaction<'a>, Vec<T>), Error> {
    tx_fetch_all(tx, entity)
}

/// Execute a custom SELECT query within a transaction and transform the result.
///
/// # Arguments
/// * `tx` - A transaction
/// * `entity` - Data object containing query parameters
/// * `to_model` - Function to transform a row into a value
///
/// # Returns
/// * `Result<(Transaction, R)>` - The transaction and transformed value, or an error
///
/// # Example
/// ```rust,no_run
/// use rusqlite::{Connection, Result};
/// use parsql::sqlite::transactional;
/// use parsql::macros::{Queryable, SqlParams};
///
/// #[derive(Queryable, SqlParams)]
/// #[table("users")]
/// #[where_clause("email LIKE ?")]
/// struct CountUsers {
///     email: String,
/// }
///
/// fn main() -> Result<()> {
///     let conn = Connection::open("test.db")?;
///     let tx = transactional::begin(&conn)?;
///     
///     let param = CountUsers {
///         email: "%example.com".to_string(),
///     };
///     
///     let (tx, count): (_, i64) = transactional::tx_select(
///         tx,
///         &param,
///         |row| row.get(0)
///     )?;
///     
///     println!("Number of users: {}", count);
///     
///     tx.commit()?;
///     Ok(())
/// }
/// ```
pub fn tx_select<'a, T, F, R>(
    tx: Transaction<'a>,
    entity: &T,
    to_model: F,
) -> Result<(Transaction<'a>, R), Error>
where
    T: SqlQuery + SqlParams,
    F: Fn(&rusqlite::Row) -> Result<R, Error>,
{
    let result = tx.select(entity, to_model)?;
    Ok((tx, result))
}

/// Execute a custom SELECT query within a transaction and transform all results.
///
/// # Arguments
/// * `tx` - A transaction
/// * `entity` - Data object containing query parameters
/// * `to_model` - Function to transform rows into values
///
/// # Returns
/// * `Result<(Transaction, Vec<R>)>` - The transaction and transformed values, or an error
///
/// # Example
/// ```rust,no_run
/// use rusqlite::{Connection, Result};
/// use parsql::sqlite::transactional;
/// use parsql::macros::{Queryable, SqlParams};
///
/// #[derive(Queryable, SqlParams)]
/// #[table("users")]
/// #[where_clause("email LIKE ?")]
/// struct GetUserNames {
///     email: String,
/// }
///
/// fn main() -> Result<()> {
///     let conn = Connection::open("test.db")?;
///     let tx = transactional::begin(&conn)?;
///     
///     let param = GetUserNames {
///         email: "%example.com".to_string(),
///     };
///     
///     let (tx, names): (_, Vec<String>) = transactional::tx_select_all(
///         tx,
///         &param,
///         |row| row.get(0)
///     )?;
///     
///     for name in names {
///         println!("User name: {}", name);
///     }
///     
///     tx.commit()?;
///     Ok(())
/// }
/// ```
pub fn tx_select_all<'a, T, F, R>(
    tx: Transaction<'a>,
    entity: &T,
    to_model: F,
) -> Result<(Transaction<'a>, Vec<R>), Error>
where
    T: SqlQuery + SqlParams,
    F: Fn(&rusqlite::Row) -> Result<R, Error>,
{
    let results = tx.select_all(entity, to_model)?;
    Ok((tx, results))
} 