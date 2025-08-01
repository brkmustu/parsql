use deadpool_postgres::{Pool, PoolError};
use postgres::types::FromSqlOwned;
//use postgres::types::FromSql;
use crate::traits::{CrudOps, FromRow, SqlCommand, SqlParams, SqlQuery, UpdateParams};
use tokio_postgres::{types::FromSql, Error, Row};

// Daha basit bir yaklaşım: PoolError'dan genel bir Error oluştur
fn pool_err_to_io_err(e: PoolError) -> Error {
    // Bu özel fonksiyon tokio_postgres'in sağladığı timeout hatasını döndürür
    // Güzel bir çözüm değil, ama çalışır bir örnek için kullanılabilir
    let err = Error::__private_api_timeout();

    // Debug süreci için stderr'e hatayı yazdıralım
    eprintln!("Pool bağlantı hatası: {}", e);

    err
}

/// # insert
///
/// Deadpool bağlantı havuzunu kullanarak veritabanına yeni bir kayıt ekler.
///
/// ## Parametreler
/// - `pool`: Deadpool bağlantı havuzu
/// - `entity`: Eklenecek veri nesnesi (SqlQuery ve SqlParams trait'lerini uygulamalıdır)
///
/// ## Dönüş Değeri
/// - `Result<i64, Error>`: Başarılı olursa, eklenen kayıt ID'sini döndürür; başarısız olursa, Error döndürür
///
/// ## Yapı Tanımı
/// Bu fonksiyonla kullanılan yapılar aşağıdaki derive makrolarıyla işaretlenmelidir:
///
/// ```rust,no_run
/// #[derive(Insertable, SqlParams)]  // Gerekli makrolar
/// #[table("tablo_adi")]            // Ekleme yapılacak tablo adı
/// pub struct VeriModeli {
///     pub alan1: String,
///     pub alan2: i32,
///     // ...
/// }
/// ```
///
/// - `Insertable`: Otomatik olarak SQL INSERT ifadeleri oluşturur
/// - `SqlParams`: Otomatik olarak SQL parametreleri oluşturur
/// - `#[table("tablo_adi")]`: Ekleme yapılacak tablo adını belirtir
///
/// ## Kullanım Örneği
/// ```rust,no_run
/// use deadpool_postgres::{Config, Runtime, Pool};
/// use tokio_postgres::{NoTls, Error};
/// use parsql::tokio_postgres::pool_crud_ops::insert;
///
/// #[derive(Insertable, SqlParams)]
/// #[table("users")]
/// pub struct InsertUser {
///     pub name: String,
///     pub email: String,
///     pub state: i16,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let mut cfg = Config::new();
///     cfg.host = Some("localhost".to_string());
///     cfg.dbname = Some("test".to_string());
///     
///     let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
///
///     let insert_user = InsertUser {
///         name: "John".to_string(),
///         email: "john@example.com".to_string(),
///         state: 1_i16,
///     };
///
///     let insert_result = insert(&pool, insert_user).await?;
///     println!("Insert result: {:?}", insert_result);
///     Ok(())
/// }
/// ```
// pub async fn insert<T: SqlQuery + SqlParams, P:for<'a> FromSql<'a> + Send + Sync>(
//     pool: &Pool,
//     entity: T,
// ) -> Result<P, Error> {
//     let client = pool.get().await.map_err(pool_err_to_io_err)?;
//     let sql = T::query();

//     if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
//         println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
//     }

//     let params = entity.params();
//     let row = client.query_one(&sql, &params).await?;
//     row.try_get::<_, P>(0)
// }

pub async fn insert<T, P>(pool: &Pool, entity: T) -> Result<P, Error>
where
    T: SqlCommand + SqlParams,
    P: FromSqlOwned + Send + Sync,
{
    let client = pool.get().await.map_err(pool_err_to_io_err)?;
    let sql = T::query();

    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let row = client.query_one(&sql, &params).await?;
    row.try_get::<_, P>(0)
}

/// # update
///
/// Deadpool bağlantı havuzunu kullanarak veritabanındaki mevcut bir kaydı günceller.
///
/// ## Parametreler
/// - `pool`: Deadpool bağlantı havuzu
/// - `entity`: Güncelleme bilgilerini içeren veri nesnesi (SqlQuery ve UpdateParams trait'lerini uygulamalıdır)
///
/// ## Dönüş Değeri
/// - `Result<bool, Error>`: Başarılı olursa, true döndürür; başarısız olursa, Error döndürür
///
/// ## Yapı Tanımı
/// Bu fonksiyonla kullanılan yapılar aşağıdaki derive makrolarıyla işaretlenmelidir:
///
/// ```rust,no_run
/// #[derive(Updateable, UpdateParams)]  // Gerekli makrolar
/// #[table("tablo_adi")]               // Güncellenecek tablo adı
/// #[update("alan1, alan2")]          // Güncellenecek alanlar (isteğe bağlı)
/// #[where_clause("id = $")]            // Güncelleme koşulu
/// pub struct VeriModeli {
///     pub id: i32,                     // Koşulda kullanılan alanlar
///     pub alan1: String,              // Güncellenecek alanlar
///     pub alan2: i32,                 // Güncellenecek alanlar
///     // ...
/// }
/// ```
///
/// - `Updateable`: Otomatik olarak SQL UPDATE ifadeleri oluşturur
/// - `UpdateParams`: Otomatik olarak güncelleme parametreleri oluşturur
/// - `#[table("tablo_adi")]`: Güncellenecek tablo adını belirtir
/// - `#[update("alan1, alan2")]`: Hangi alanların güncelleneceğini belirtir (belirtilmezse, tüm alanlar güncellenir)
/// - `#[where_clause("id = $")]`: Güncelleme koşulunu belirtir (`$` parametre değeri ile değiştirilir)
///
/// ## Kullanım Örneği
/// ```rust,no_run
/// use deadpool_postgres::{Config, Runtime, Pool};
/// use tokio_postgres::{NoTls, Error};
/// use parsql::tokio_postgres::pool_crud_ops::update;
///
/// #[derive(Updateable, UpdateParams)]
/// #[table("users")]
/// #[update("name, email")]
/// #[where_clause("id = $")]
/// pub struct UpdateUser {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
///     pub state: i16,  // update özniteliğinde belirtilmediği için bu alan güncellenmez
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let mut cfg = Config::new();
///     cfg.host = Some("localhost".to_string());
///     cfg.dbname = Some("test".to_string());
///     
///     let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
///
///     let update_user = UpdateUser {
///         id: 1,
///         name: String::from("John"),
///         email: String::from("john@example.com"),
///         state: 2,
///     };
///
///     let update_result = update(&pool, update_user).await?;
///     println!("Update result: {:?}", update_result);
///     Ok(())
/// }
/// ```
pub async fn update<T: SqlCommand + UpdateParams>(pool: &Pool, entity: T) -> Result<bool, Error> {
    let client = pool.get().await.map_err(pool_err_to_io_err)?;
    let sql = T::query();

    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
    }

    let params = entity.params();
    match client.execute(&sql, &params).await {
        Ok(_) => Ok(true),
        Err(e) => Err(e),
    }
}

/// # delete
///
/// Deadpool bağlantı havuzunu kullanarak veritabanından bir kaydı siler.
///
/// ## Parametreler
/// - `pool`: Deadpool bağlantı havuzu
/// - `entity`: Silme bilgilerini içeren veri nesnesi (SqlQuery ve SqlParams trait'lerini uygulamalıdır)
///
/// ## Dönüş Değeri
/// - `Result<u64, Error>`: Başarılı olursa, silinen kayıt sayısını döndürür; başarısız olursa, Error döndürür
///
/// ## Yapı Tanımı
/// Bu fonksiyonla kullanılan yapılar aşağıdaki derive makrolarıyla işaretlenmelidir:
///
/// ```rust,no_run
/// #[derive(Deletable, SqlParams)]   // Gerekli makrolar
/// #[table("tablo_adi")]             // Silinecek tablo adı
/// #[where_clause("id = $")]          // Silme koşulu
/// pub struct VeriModeli {
///     pub id: i32,                   // Koşulda kullanılan alanlar
///     // Diğer alanlar eklenebilir, ancak genellikle sadece koşul alanları gereklidir
/// }
/// ```
///
/// - `Deletable`: Otomatik olarak SQL DELETE ifadeleri oluşturur
/// - `SqlParams`: Otomatik olarak SQL parametreleri oluşturur
/// - `#[table("tablo_adi")]`: Silinecek tablo adını belirtir
/// - `#[where_clause("id = $")]`: Silme koşulunu belirtir (`$` parametre değeri ile değiştirilir)
///
/// ## Kullanım Örneği
/// ```rust,no_run
/// use deadpool_postgres::{Config, Runtime, Pool};
/// use tokio_postgres::{NoTls, Error};
/// use parsql::tokio_postgres::pool_crud_ops::delete;
///
/// #[derive(Deletable, SqlParams)]
/// #[table("users")]
/// #[where_clause("id = $")]
/// pub struct DeleteUser {
///     pub id: i32,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let mut cfg = Config::new();
///     cfg.host = Some("localhost".to_string());
///     cfg.dbname = Some("test".to_string());
///     
///     let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
///
///     let delete_user = DeleteUser { id: 6 };
///     let delete_result = delete(&pool, delete_user).await?;
///     
///     println!("Delete result: {:?}", delete_result);
///     Ok(())
/// }
/// ```
pub async fn delete<T: SqlCommand + SqlParams>(pool: &Pool, entity: T) -> Result<u64, Error> {
    let client = pool.get().await.map_err(pool_err_to_io_err)?;
    let sql = T::query();

    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
    }

    let params = entity.params();
    client.execute(&sql, &params).await
}

/// # fetch
///
/// Deadpool bağlantı havuzunu kullanarak veritabanından bir kaydı alır.
///
/// ## Parametreler
/// - `pool`: Deadpool bağlantı havuzu
/// - `params`: Sorgu parametrelerini içeren veri nesnesi (SqlQuery, FromRow ve SqlParams trait'lerini uygulamalıdır)
///
/// ## Dönüş Değeri
/// - `Result<T, Error>`: Başarılı olursa, alınan kaydı döndürür; başarısız olursa, Error döndürür
///
/// ## Yapı Tanımı
/// Bu fonksiyonla kullanılan yapılar aşağıdaki derive makrolarıyla işaretlenmelidir:
///
/// ```rust,no_run
/// #[derive(Queryable, SqlParams, FromRow)]   // Gerekli makrolar
/// #[table("tablo_adi")]                     // Sorgulanacak tablo adı
/// #[where_clause("id = $")]                  // Sorgu koşulu
/// pub struct VeriModeli {
///     pub id: i32,                          // Koşulda kullanılan alanlar
///     pub alan1: String,                    // Getirilen veri alanları
///     pub alan2: i32,                       // Getirilen veri alanları
///     // ...
/// }
/// ```
///
/// - `Queryable`: Otomatik olarak SQL SELECT ifadeleri oluşturur
/// - `SqlParams`: Otomatik olarak SQL parametreleri oluşturur
/// - `FromRow`: Veritabanı satırını yapıya dönüştürür
/// - `#[table("tablo_adi")]`: Sorgulanacak tablo adını belirtir
/// - `#[where_clause("id = $")]`: Sorgu koşulunu belirtir (`$` parametre değeri ile değiştirilir)
///
/// ## Kullanım Örneği
/// ```rust,no_run
/// use deadpool_postgres::{Config, Runtime, Pool};
/// use tokio_postgres::{NoTls, Error};
/// use parsql::tokio_postgres::pool_crud_ops::fetch;
///
/// #[derive(Queryable, SqlParams, FromRow)]
/// #[table("users")]
/// #[where_clause("id = $")]
/// pub struct GetUser {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
///     pub state: i16,
/// }
///
/// impl GetUser {
///     pub fn new(id: i32) -> Self {
///         Self {
///             id,
///             name: String::new(),
///             email: String::new(),
///             state: 0,
///         }
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let mut cfg = Config::new();
///     cfg.host = Some("localhost".to_string());
///     cfg.dbname = Some("test".to_string());
///     
///     let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
///
///     let user_params = GetUser::new(1);
///     let user = fetch(&pool, &user_params).await?;
///     
///     println!("User: {:?}", user);
///     Ok(())
/// }
/// ```
pub async fn fetch<P, R>(pool: &Pool, params: &P) -> Result<R, Error>
where
    P: SqlQuery<R> + SqlParams,
    R: FromRow,
{
    let client = pool.get().await.map_err(pool_err_to_io_err)?;
    let sql = P::query();

    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
    }

    let query_params = params.params();
    let row = client.query_one(&sql, &query_params).await?;
    R::from_row(&row)
}

/// # fetch_all
///
/// Deadpool bağlantı havuzunu kullanarak veritabanından birden fazla kaydı alır.
///
/// ## Parametreler
/// - `pool`: Deadpool bağlantı havuzu
/// - `params`: Sorgu parametrelerini içeren veri nesnesi (SqlQuery, FromRow ve SqlParams trait'lerini uygulamalıdır)
///
/// ## Dönüş Değeri
/// - `Result<Vec<T>, Error>`: Başarılı olursa, alınan kayıtları içeren bir vektör döndürür; başarısız olursa, Error döndürür
///
/// ## Yapı Tanımı
/// Bu fonksiyonla kullanılan yapılar aşağıdaki derive makrolarıyla işaretlenmelidir:
///
/// ```rust,no_run
/// #[derive(Queryable, SqlParams, FromRow)]   // Gerekli makrolar
/// #[table("tablo_adi")]                     // Sorgulanacak tablo adı
/// #[where_clause("state = $")]              // Sorgu koşulu
/// pub struct VeriModeli {
///     pub id: i32,                          // Alınacak alanlar
///     pub alan1: String,                    // Alınacak alanlar
///     pub alan2: i32,                       // Alınacak alanlar
///     pub state: i16,                       // Koşulda kullanılan alanlar
///     // ...
/// }
/// ```
///
/// - `Queryable`: Otomatik olarak SQL SELECT ifadeleri oluşturur
/// - `SqlParams`: Otomatik olarak SQL parametreleri oluşturur
/// - `FromRow`: Veritabanı satırını yapıya dönüştürür
/// - `#[table("tablo_adi")]`: Sorgulanacak tablo adını belirtir
/// - `#[where_clause("state = $")]`: Sorgu koşulunu belirtir (`$` parametre değeri ile değiştirilir)
///
/// ## Kullanım Örneği
/// ```rust,no_run
/// use deadpool_postgres::{Config, Runtime, Pool};
/// use tokio_postgres::{NoTls, Error};
/// use parsql::tokio_postgres::pool_crud_ops::fetch_all;
///
/// #[derive(Queryable, SqlParams, FromRow)]
/// #[table("users")]
/// #[where_clause("state = $")]
/// pub struct ListUsers {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
///     pub state: i16,
/// }
///
/// impl ListUsers {
///     pub fn new(state: i16) -> Self {
///         Self {
///             id: 0,
///             name: String::new(),
///             email: String::new(),
///             state,
///         }
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let mut cfg = Config::new();
///     cfg.host = Some("localhost".to_string());
///     cfg.dbname = Some("test".to_string());
///     
///     let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
///
///     let user_params = ListUsers::new(1);
///     let users = fetch_all(&pool, &user_params).await?;
///     
///     println!("Users: {:?}", users);
///     Ok(())
/// }
/// ```
pub async fn fetch_all<P, R>(pool: &Pool, params: &P) -> Result<Vec<R>, Error>
where
    P: SqlQuery<R> + SqlParams,
    R: FromRow,
{
    let client = pool.get().await.map_err(pool_err_to_io_err)?;
    let sql = P::query();

    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
    }

    let query_params = params.params();
    let rows = client.query(&sql, &query_params).await?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        results.push(R::from_row(&row)?);
    }

    Ok(results)
}

/// # select
///
/// Deadpool bağlantı havuzunu kullanarak özel bir model dönüştürücü fonksiyon ile veritabanından bir kayıt seçer.
///
/// ## Parametreler
/// - `pool`: Deadpool bağlantı havuzu
/// - `entity`: Sorgu parametrelerini içeren veri nesnesi (SqlQuery ve SqlParams trait'lerini uygulamalıdır)
/// - `to_model`: Satırı modele dönüştüren fonksiyon
///
/// ## Dönüş Değeri
/// - `Result<R, Error>`: Başarılı olursa, dönüştürülen modeli döndürür; başarısız olursa, Error döndürür
///
/// ## Kullanım Örneği
/// ```rust,no_run
/// use deadpool_postgres::{Config, Runtime, Pool};
/// use tokio_postgres::{NoTls, Error, Row};
/// use parsql::tokio_postgres::pool_crud_ops::select;
///
/// #[derive(Queryable, SqlParams)]
/// #[table("users")]
/// #[where_clause("id = $")]
/// pub struct UserQuery {
///     pub id: i32,
/// }
///
/// pub struct UserModel {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
///     pub is_active: bool,
/// }
///
/// impl UserQuery {
///     pub fn new(id: i32) -> Self {
///         Self { id }
///     }
/// }
///
/// fn row_to_user(row: &Row) -> Result<UserModel, Error> {
///     Ok(UserModel {
///         id: row.try_get("id")?,
///         name: row.try_get("name")?,
///         email: row.try_get("email")?,
///         is_active: row.try_get::<_, i16>("state")? == 1,
///     })
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let mut cfg = Config::new();
///     cfg.host = Some("localhost".to_string());
///     cfg.dbname = Some("test".to_string());
///     
///     let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
///
///     let query = UserQuery::new(1);
///     let user = select(&pool, query, row_to_user).await?;
///     
///     println!("User: {:?}", user);
///     Ok(())
/// }
/// ```
pub async fn select<T: SqlQuery<T> + SqlParams, R, F>(
    pool: &Pool,
    entity: T,
    to_model: F,
) -> Result<R, Error>
where
    F: Fn(&Row) -> Result<R, Error>,
{
    let client = pool.get().await.map_err(pool_err_to_io_err)?;
    let sql = T::query();

    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let row = client.query_one(&sql, &params).await?;
    to_model(&row)
}

/// # select_all
///
/// Deadpool bağlantı havuzunu kullanarak özel bir model dönüştürücü fonksiyon ile veritabanından birden fazla kayıt seçer.
///
/// ## Parametreler
/// - `pool`: Deadpool bağlantı havuzu
/// - `entity`: Sorgu parametrelerini içeren veri nesnesi (SqlQuery ve SqlParams trait'lerini uygulamalıdır)
/// - `to_model`: Satırı modele dönüştüren fonksiyon
///
/// ## Dönüş Değeri
/// - `Result<Vec<R>, Error>`: Başarılı olursa, dönüştürülen modelleri içeren bir vektör döndürür; başarısız olursa, Error döndürür
///
/// ## Kullanım Örneği
/// ```rust,no_run
/// use deadpool_postgres::{Config, Runtime, Pool};
/// use tokio_postgres::{NoTls, Error, Row};
/// use parsql::tokio_postgres::pool_crud_ops::select_all;
///
/// #[derive(Queryable, SqlParams)]
/// #[table("users")]
/// #[where_clause("state = $")]
/// pub struct UsersQuery {
///     pub state: i16,
/// }
///
/// pub struct UserModel {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
///     pub is_active: bool,
/// }
///
/// impl UsersQuery {
///     pub fn new(state: i16) -> Self {
///         Self { state }
///     }
/// }
///
/// fn row_to_user(row: &Row) -> UserModel {
///     UserModel {
///         id: row.get("id"),
///         name: row.get("name"),
///         email: row.get("email"),
///         is_active: row.get::<_, i16>("state") == 1,
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let mut cfg = Config::new();
///     cfg.host = Some("localhost".to_string());
///     cfg.dbname = Some("test".to_string());
///     
///     let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
///
///     let query = UsersQuery::new(1);
///     let users = select_all(&pool, query, row_to_user).await?;
///     
///     println!("Users: {:?}", users);
///     Ok(())
/// }
/// ```
pub async fn select_all<T: SqlQuery<T> + SqlParams, R, F>(
    pool: &Pool,
    entity: T,
    to_model: F,
) -> Result<Vec<R>, Error>
where
    F: Fn(&Row) -> R,
{
    let client = pool.get().await.map_err(pool_err_to_io_err)?;
    let sql = T::query();

    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
    }

    let params = entity.params();
    let rows = client.query(&sql, &params).await?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        results.push(to_model(&row));
    }

    Ok(results)
}

// Deprecated functions for backward compatibility
#[deprecated(
    since = "0.2.0",
    note = "Renamed to `fetch`. Please use `fetch` function instead."
)]
pub async fn get<T: SqlQuery<T> + FromRow + SqlParams>(
    pool: &Pool,
    params: &T,
) -> Result<T, Error> {
    fetch(pool, params).await
}

#[deprecated(
    since = "0.2.0",
    note = "Renamed to `fetch_all`. Please use `fetch_all` function instead."
)]
pub async fn get_all<T: SqlQuery<T> + FromRow + SqlParams>(
    pool: &Pool,
    params: &T,
) -> Result<Vec<T>, Error> {
    fetch_all(pool, params).await
}

/// CrudOps trait implementation for deadpool_postgres::Client
#[async_trait::async_trait]
impl CrudOps for Pool {
    async fn insert<T, P: for<'a> FromSql<'a> + Send + Sync>(&self, entity: T) -> Result<P, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync,
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;
        let sql = T::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-DEADPOOL-POSTGRES] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let row = client.query_one(&sql, &params).await?;
        row.try_get::<_, P>(0)
    }

    async fn update<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + UpdateParams + Send + Sync,
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;
        let sql = T::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-DEADPOOL-POSTGRES] Execute SQL: {}", sql);
        }

        let params = entity.params();
        client.execute(&sql, &params).await
    }

    async fn delete<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlCommand + SqlParams + Send + Sync,
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;
        let sql = T::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-DEADPOOL-POSTGRES] Execute SQL: {}", sql);
        }

        let params = entity.params();
        client.execute(&sql, &params).await
    }

    async fn fetch<P, R>(&self, params: &P) -> Result<R, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync,
        R: FromRow + Send + Sync,
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;
        let sql = P::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-DEADPOOL-POSTGRES] Execute SQL: {}", sql);
        }

        let query_params = params.params();
        let row = client.query_one(&sql, &query_params).await?;
        R::from_row(&row)
    }

    async fn fetch_all<P, R>(&self, params: &P) -> Result<Vec<R>, Error>
    where
        P: SqlQuery<R> + SqlParams + Send + Sync,
        R: FromRow + Send + Sync,
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;
        let sql = P::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-DEADPOOL-POSTGRES] Execute SQL: {}", sql);
        }

        let query_params = params.params();
        let rows = client.query(&sql, &query_params).await?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(R::from_row(&row)?);
        }

        Ok(results)
    }

    async fn select<T, R, F>(&self, entity: T, to_model: F) -> Result<R, Error>
    where
        T: SqlQuery<T> + SqlParams + Send + Sync,
        F: FnOnce(&Row) -> Result<R, Error> + Send + Sync,
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;
        let sql = T::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-DEADPOOL-POSTGRES] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let row = client.query_one(&sql, &params).await?;
        to_model(&row)
    }

    async fn select_all<T, R, F>(&self, entity: T, to_model: F) -> Result<Vec<R>, Error>
    where
        T: SqlQuery<T> + SqlParams + Send + Sync,
        F: Fn(&Row) -> R + Send + Sync,
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;
        let sql = T::query();

        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-DEADPOOL-POSTGRES] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let rows = client.query(&sql, &params).await?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(to_model(&row));
        }

        Ok(results)
    }
}
