use postgres;
use postgres::{
    types::{FromSql, ToSql},
    Error, Row,
};

/// SQL sorguları oluşturmak için trait (SELECT işlemleri için).
/// Bu trait, `Queryable` derive makrosu tarafından uygulanır.
pub trait SqlQuery<R> {
    /// SQL sorgu string'ini döndürür.
    fn query() -> String;
}

/// SQL komutları oluşturmak için trait (INSERT/UPDATE/DELETE işlemleri için).
/// Bu trait, `Insertable`, `Updateable` ve `Deletable` derive makroları tarafından uygulanır.
pub trait SqlCommand {
    /// SQL komut string'ini döndürür.
    fn query() -> String;
}

/// SQL parametreleri sağlamak için trait.
/// Bu trait, `SqlParams` derive makrosu tarafından uygulanır.
pub trait SqlParams {
    /// SQL parametrelerinin referanslarını içeren bir vektör döndürür.
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

/// UPDATE işlemleri için parametre sağlamak üzere trait.
/// Bu trait, `UpdateParams` derive makrosu tarafından uygulanır.
pub trait UpdateParams {
    /// UPDATE işlemleri için SQL parametrelerinin referanslarını içeren bir vektör döndürür.
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

/// Veritabanı satırlarını Rust struct'larına dönüştürmek için trait.
/// Bu trait, `FromRow` derive makrosu tarafından uygulanır.
pub trait FromRow {
    /// Bir veritabanı satırını Rust struct'ına dönüştürür.
    ///
    /// # Argümanlar
    /// * `row` - Veritabanı satırına referans
    ///
    /// # Dönüş Değeri
    /// * `Result<Self, Error>` - Dönüştürülmüş struct veya hata
    fn from_row(row: &Row) -> Result<Self, Error>
    where
        Self: Sized;
}

/// CrudOps trait defines the CRUD (Create, Read, Update, Delete) operations
/// that can be performed on a PostgreSQL database.
///
/// This trait is implemented for the `postgres::Client` struct, allowing
/// CRUD operations to be called as extension methods on a client.
///
/// # Example
///
/// ```rust,no_run
/// use postgres::{Client, NoTls, Error};
/// use parsql::postgres::CrudOps;
/// use parsql::macros::{Insertable, SqlParams, Queryable, FromRow};
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
/// #[where_clause("id = $")]
/// struct GetUser {
///     id: i32,
///     name: String,
///     email: String,
/// }
///
/// fn main() -> Result<(), Error> {
///     let mut client = Client::connect(
///         "host=localhost user=postgres dbname=test",
///         NoTls,
///     )?;
///    
///     // Extension method for insert
///     let insert_user = InsertUser {
///         name: "John".to_string(),
///         email: "john@example.com".to_string(),
///     };
///     let rows_affected = client.insert(insert_user)?;
///    
///     // Extension method for get
///     let get_user = GetUser {
///         id: 1,
///         name: String::new(),
///         email: String::new(),
///     };
///     let user = client.fetch(&get_user)?;
///    
///     println!("User: {:?}", user);
///     Ok(())
/// }
/// ```
pub trait CrudOps {
    /// Inserts a new record into the PostgreSQL database.
    ///
    /// # Arguments
    /// * `entity` - Data object to be inserted (must implement SqlCommand and SqlParams traits)
    ///
    /// # Returns
    /// * `Result<P, Error>` - On success, returns the inserted ID or return value; on failure, returns Error
    fn insert<T: SqlCommand + SqlParams, P: for<'a> FromSql<'a> + Send + Sync>(
        &mut self,
        entity: T,
    ) -> Result<P, Error>;

    /// Updates records in the PostgreSQL database.
    ///
    /// # Arguments
    /// * `entity` - Data object containing the update information (must implement SqlCommand and UpdateParams traits)
    ///
    /// # Returns
    /// * `Result<u64, Error>` - On success, returns the number of updated records; on failure, returns Error
    fn update<T: SqlCommand + UpdateParams>(&mut self, entity: T) -> Result<u64, Error>;

    /// Deletes records from the PostgreSQL database.
    ///
    /// # Arguments
    /// * `entity` - Data object containing delete conditions (must implement SqlCommand and SqlParams traits)
    ///
    /// # Returns
    /// * `Result<u64, Error>` - On success, returns the number of deleted records; on failure, returns Error
    fn delete<T: SqlCommand + SqlParams>(&mut self, entity: T) -> Result<u64, Error>;

    /// Retrieves a single record from the PostgreSQL database.
    ///
    /// # Arguments
    /// * `params` - Data object containing query parameters (must implement SqlQuery and SqlParams traits)
    ///
    /// # Returns
    /// * `Result<R, Error>` - On success, returns the retrieved record; on failure, returns Error
    fn fetch<P, R>(&mut self, params: &P) -> Result<R, Error>
    where
        P: SqlQuery<R> + SqlParams,
        R: FromRow;

    /// Retrieves multiple records from the PostgreSQL database.
    ///
    /// # Arguments
    /// * `params` - Data object containing query parameters (must implement SqlQuery and SqlParams traits)
    ///
    /// # Returns
    /// * `Result<Vec<R>, Error>` - On success, returns a vector of records; on failure, returns Error
    fn fetch_all<P, R>(&mut self, params: &P) -> Result<Vec<R>, Error>
    where
        P: SqlQuery<R> + SqlParams,
        R: FromRow;

    /// Executes a custom query and transforms the result using the provided function.
    ///
    /// # Arguments
    /// * `entity` - Data object containing query parameters (must implement SqlQuery and SqlParams traits)
    /// * `to_model` - Function to transform the database row into the desired type
    ///
    /// # Returns
    /// * `Result<R, Error>` - On success, returns the transformed result; on failure, returns Error
    fn select<T, F, R>(&mut self, entity: &T, to_model: F) -> Result<R, Error>
    where
        T: SqlQuery<T> + SqlParams,
        F: FnOnce(&Row) -> Result<R, Error>;

    /// Executes a custom query and transforms all results using the provided function.
    ///
    /// # Arguments
    /// * `entity` - Data object containing query parameters (must implement SqlQuery and SqlParams traits)
    /// * `to_model` - Function to transform database rows into the desired type
    ///
    /// # Returns
    /// * `Result<Vec<R>, Error>` - On success, returns a vector of transformed results; on failure, returns Error
    fn select_all<T, F, R>(&mut self, entity: &T, to_model: F) -> Result<Vec<R>, Error>
    where
        T: SqlQuery<T> + SqlParams,
        F: FnMut(&Row) -> Result<R, Error>;
}
