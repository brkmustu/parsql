use postgres::{Transaction, Error, Row};
use crate::{SqlQuery, SqlParams, FromRow, UpdateParams, CrudOps};

/// CrudOps trait implementasyonu Transaction<'_> için.
/// Bu sayede transaction içinde tüm CRUD işlemleri extension metotları olarak kullanılabilir.
impl<'a> CrudOps for Transaction<'a> {
    fn insert<T: SqlQuery + SqlParams>(&mut self, entity: T) -> Result<u64, Error> {
        let sql = T::query();
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-POSTGRES] Execute SQL (Transaction): {}", sql);
        }

        let params = entity.params();
        self.execute(&sql, &params)
    }

    fn update<T: SqlQuery + UpdateParams>(&mut self, entity: T) -> Result<u64, Error> {
        let sql = T::query();
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-POSTGRES] Execute SQL (Transaction): {}", sql);
        }

        let params = entity.params();
        self.execute(&sql, &params)
    }

    fn delete<T: SqlQuery + SqlParams>(&mut self, entity: T) -> Result<u64, Error> {
        let sql = T::query();
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-POSTGRES] Execute SQL (Transaction): {}", sql);
        }

        let params = entity.params();
        self.execute(&sql, &params)
    }

    fn fetch<T: SqlQuery + FromRow + SqlParams>(&mut self, entity: &T) -> Result<T, Error> {
        let sql = T::query();
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-POSTGRES] Execute SQL (Transaction): {}", sql);
        }
        
        let params = entity.params();
        let row = self.query_one(&sql, &params)?;
        T::from_row(&row)
    }

    fn fetch_all<T: SqlQuery + FromRow + SqlParams>(&mut self, entity: &T) -> Result<Vec<T>, Error> {
        let sql = T::query();
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-POSTGRES] Execute SQL (Transaction): {}", sql);
        }
        
        let params = entity.params();
        let rows = self.query(&sql, &params)?;
        
        rows.iter()
            .map(|row| T::from_row(row))
            .collect::<Result<Vec<_>, _>>()
    }

    fn select<T, F, R>(&mut self, entity: &T, to_model: F) -> Result<R, Error>
    where
        T: SqlQuery + SqlParams,
        F: FnOnce(&Row) -> Result<R, Error>,
    {
        let sql = T::query();
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-POSTGRES] Execute SQL (Transaction): {}", sql);
        }

        let params = entity.params();
        let row = self.query_one(&sql, &params)?;
        to_model(&row)
    }

    fn select_all<T, F, R>(&mut self, entity: &T, to_model: F) -> Result<Vec<R>, Error>
    where
        T: SqlQuery + SqlParams,
        F: FnMut(&Row) -> Result<R, Error>,
    {
        let sql = T::query();
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-POSTGRES] Execute SQL (Transaction): {}", sql);
        }

        let params = entity.params();
        let rows = self.query(&sql, &params)?;
        
        rows.iter().map(to_model).collect()
    }
}

/// # begin
/// 
/// Yeni bir transaction başlatır.
/// 
/// ## Parametreler
/// - `client`: Veritabanı bağlantı istemcisi
/// 
/// ## Dönüş Değeri
/// - `Result<Transaction<'_>, Error>`: Başarılı olursa, transaction nesnesini döner; hata durumunda Error döner
/// 
/// ## Örnek Kullanım
/// ```rust,no_run
/// use postgres::{Client, NoTls, Error};
/// use parsql::postgres::transactional::begin;
/// 
/// fn main() -> Result<(), Error> {
///     let mut client = Client::connect(
///         "host=localhost user=postgres dbname=test",
///         NoTls,
///     )?;
///     
///     let tx = begin(&mut client)?;
///     
///     // İşlemler...
///     
///     // Transaction'ı tamamla
///     tx.commit()?;
///     Ok(())
/// }
/// ```
pub fn begin<'a>(client: &'a mut postgres::Client) -> Result<Transaction<'a>, Error> {
    client.transaction()
}

/// # tx_insert
/// 
/// Transaction içinde bir kaydı veritabanına ekler.
/// 
/// ## Parametreler
/// - `tx`: Transaction nesnesi
/// - `entity`: Eklenecek veri nesnesi (SqlQuery ve SqlParams trait'lerini implement etmeli)
/// 
/// ## Dönüş Değeri
/// - `Result<(Transaction<'_>, u64), Error>`: Başarılı olursa, transaction ve etkilenen kayıt sayısını döner; hata durumunda Error döner
/// 
/// ## Örnek Kullanım
/// ```rust,no_run
/// use postgres::{Client, NoTls, Error};
/// use parsql::postgres::transactional::{begin, tx_insert};
/// 
/// #[derive(Insertable, SqlParams)]
/// #[table("users")]
/// pub struct InsertUser {
///     pub name: String,
///     pub email: String,
/// }
///
/// fn main() -> Result<(), Error> {
///     let mut client = Client::connect(
///         "host=localhost user=postgres dbname=test",
///         NoTls,
///     )?;
///     
///     let mut tx = begin(&mut client)?;
///     
///     let insert_user = InsertUser {
///         name: "John".to_string(),
///         email: "john@example.com".to_string(),
///     };
///     
///     let (tx, rows_affected) = tx_insert(tx, insert_user)?;
///     
///     // İşlemler devam edebilir...
///     
///     // Transaction'ı tamamla
///     tx.commit()?;
///     Ok(())
/// }
/// ```
pub fn tx_insert<'a, T>(mut tx: Transaction<'a>, entity: T) -> Result<(Transaction<'a>, u64), Error>
where
    T: SqlQuery + SqlParams,
{
    let result = tx.insert(entity)?;
    Ok((tx, result))
}

/// # tx_update
/// 
/// Transaction içinde bir kaydı günceller.
/// 
/// ## Parametreler
/// - `tx`: Transaction nesnesi
/// - `entity`: Güncellenecek veri nesnesi (SqlQuery ve UpdateParams trait'lerini implement etmeli)
/// 
/// ## Dönüş Değeri
/// - `Result<(Transaction<'_>, u64), Error>`: Başarılı olursa, transaction ve etkilenen kayıt sayısını döner; hata durumunda Error döner
/// 
/// ## Örnek Kullanım
/// ```rust,no_run
/// use postgres::{Client, NoTls, Error};
/// use parsql::postgres::transactional::{begin, tx_update};
/// 
/// #[derive(Updateable, UpdateParams)]
/// #[table("users")]
/// #[update("name, email")]
/// #[where_clause("id = $")]
/// pub struct UpdateUser {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
/// }
///
/// fn main() -> Result<(), Error> {
///     let mut client = Client::connect(
///         "host=localhost user=postgres dbname=test",
///         NoTls,
///     )?;
///     
///     let mut tx = begin(&mut client)?;
///     
///     let update_user = UpdateUser {
///         id: 1,
///         name: "John Updated".to_string(),
///         email: "john.updated@example.com".to_string(),
///     };
///     
///     let (tx, rows_affected) = tx_update(tx, update_user)?;
///     
///     // İşlemler devam edebilir...
///     
///     // Transaction'ı tamamla
///     tx.commit()?;
///     Ok(())
/// }
/// ```
pub fn tx_update<'a, T>(mut tx: Transaction<'a>, entity: T) -> Result<(Transaction<'a>, u64), Error>
where
    T: SqlQuery + UpdateParams,
{
    let result = tx.update(entity)?;
    Ok((tx, result))
}

/// # tx_delete
/// 
/// Transaction içinde bir kaydı siler.
/// 
/// ## Parametreler
/// - `tx`: Transaction nesnesi
/// - `entity`: Silinecek veri nesnesi (SqlQuery ve SqlParams trait'lerini implement etmeli)
/// 
/// ## Dönüş Değeri
/// - `Result<(Transaction<'_>, u64), Error>`: Başarılı olursa, transaction ve etkilenen kayıt sayısını döner; hata durumunda Error döner
/// 
/// ## Örnek Kullanım
/// ```rust,no_run
/// use postgres::{Client, NoTls, Error};
/// use parsql::postgres::transactional::{begin, tx_delete};
/// 
/// #[derive(Deletable, SqlParams)]
/// #[table("users")]
/// #[where_clause("id = $")]
/// pub struct DeleteUser {
///     pub id: i32,
/// }
///
/// fn main() -> Result<(), Error> {
///     let mut client = Client::connect(
///         "host=localhost user=postgres dbname=test",
///         NoTls,
///     )?;
///     
///     let mut tx = begin(&mut client)?;
///     
///     let delete_user = DeleteUser { id: 1 };
///     
///     let (tx, rows_affected) = tx_delete(tx, delete_user)?;
///     
///     // İşlemler devam edebilir...
///     
///     // Transaction'ı tamamla
///     tx.commit()?;
///     Ok(())
/// }
/// ```
pub fn tx_delete<'a, T>(mut tx: Transaction<'a>, entity: T) -> Result<(Transaction<'a>, u64), Error>
where
    T: SqlQuery + SqlParams,
{
    let result = tx.delete(entity)?;
    Ok((tx, result))
}

/// # tx_fetch
/// 
/// Transaction içinde tek bir kaydı getirir.
/// 
/// ## Parametreler
/// - `tx`: Transaction nesnesi
/// - `entity`: Sorgu parametresi nesnesi (SqlQuery, FromRow ve SqlParams trait'lerini implement etmeli)
/// 
/// ## Dönüş Değeri
/// - `Result<(Transaction<'_>, T), Error>`: Başarılı olursa, transaction ve bulunan kaydı döner; hata durumunda Error döner
/// 
/// ## Örnek Kullanım
/// ```rust,no_run
/// use postgres::{Client, NoTls, Error};
/// use parsql::postgres::transactional::{begin, tx_fetch};
/// 
/// #[derive(Queryable, FromRow, SqlParams)]
/// #[table("users")]
/// #[where_clause("id = $")]
/// pub struct GetUser {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
/// }
///
/// fn main() -> Result<(), Error> {
///     let mut client = Client::connect(
///         "host=localhost user=postgres dbname=test",
///         NoTls,
///     )?;
///     
///     let mut tx = begin(&mut client)?;
///     
///     let get_user = GetUser {
///         id: 1,
///         name: String::new(),
///         email: String::new(),
///     };
///     
///     let (tx, user) = tx_fetch(tx, &get_user)?;
///     
///     // İşlemler devam edebilir...
///     
///     // Transaction'ı tamamla
///     tx.commit()?;
///     Ok(())
/// }
/// ```
pub fn tx_fetch<'a, T>(mut tx: Transaction<'a>, entity: &T) -> Result<(Transaction<'a>, T), Error>
where
    T: SqlQuery + FromRow + SqlParams,
{
    let result = tx.fetch(entity)?;
    Ok((tx, result))
}

/// # tx_fetch_all
/// 
/// Transaction içinde birden fazla kaydı getirir.
/// 
/// ## Parametreler
/// - `tx`: Transaction nesnesi
/// - `entity`: Sorgu parametresi nesnesi (SqlQuery, FromRow ve SqlParams trait'lerini implement etmeli)
/// 
/// ## Dönüş Değeri
/// - `Result<(Transaction<'_>, Vec<T>), Error>`: Başarılı olursa, transaction ve bulunan kayıtların listesini döner; hata durumunda Error döner
/// 
/// ## Örnek Kullanım
/// ```rust,no_run
/// use postgres::{Client, NoTls, Error};
/// use parsql::postgres::transactional::{begin, tx_fetch_all};
/// 
/// #[derive(Queryable, FromRow, SqlParams)]
/// #[table("users")]
/// #[where_clause("active = $")]
/// pub struct GetUsers {
///     pub active: bool,
///     pub id: i32,
///     pub name: String,
///     pub email: String,
/// }
///
/// fn main() -> Result<(), Error> {
///     let mut client = Client::connect(
///         "host=localhost user=postgres dbname=test",
///         NoTls,
///     )?;
///     
///     let mut tx = begin(&mut client)?;
///     
///     let get_users = GetUsers {
///         active: true,
///         id: 0,
///         name: String::new(),
///         email: String::new(),
///     };
///     
///     let (tx, users) = tx_fetch_all(tx, &get_users)?;
///     
///     // İşlemler devam edebilir...
///     
///     // Transaction'ı tamamla
///     tx.commit()?;
///     Ok(())
/// }
/// ```
pub fn tx_fetch_all<'a, T>(mut tx: Transaction<'a>, entity: &T) -> Result<(Transaction<'a>, Vec<T>), Error>
where
    T: SqlQuery + FromRow + SqlParams,
{
    let result = tx.fetch_all(entity)?;
    Ok((tx, result))
}

/// # tx_select
/// 
/// Transaction içinde özel bir sorgu çalıştırır ve sonucu dönüştürür.
/// 
/// ## Parametreler
/// - `tx`: Transaction nesnesi
/// - `entity`: Sorgu parametresi nesnesi (SqlQuery ve SqlParams trait'lerini implement etmeli)
/// - `to_model`: Row nesnesini hedef nesne tipine dönüştüren fonksiyon
/// 
/// ## Dönüş Değeri
/// - `Result<(Transaction<'_>, R), Error>`: Başarılı olursa, transaction ve dönüştürülmüş nesneyi döner; hata durumunda Error döner
pub fn tx_select<'a, T, F, R>(mut tx: Transaction<'a>, entity: &T, to_model: F) -> Result<(Transaction<'a>, R), Error>
where
    T: SqlQuery + SqlParams,
    F: FnOnce(&Row) -> Result<R, Error>,
{
    let result = tx.select(entity, to_model)?;
    Ok((tx, result))
}

/// # tx_select_all
/// 
/// Transaction içinde özel bir sorgu çalıştırır ve tüm sonuçları dönüştürür.
/// 
/// ## Parametreler
/// - `tx`: Transaction nesnesi
/// - `entity`: Sorgu parametresi nesnesi (SqlQuery ve SqlParams trait'lerini implement etmeli)
/// - `to_model`: Row nesnesini hedef nesne tipine dönüştüren fonksiyon
/// 
/// ## Dönüş Değeri
/// - `Result<(Transaction<'_>, Vec<R>), Error>`: Başarılı olursa, transaction ve dönüştürülmüş nesnelerin listesini döner; hata durumunda Error döner
pub fn tx_select_all<'a, T, F, R>(mut tx: Transaction<'a>, entity: &T, to_model: F) -> Result<(Transaction<'a>, Vec<R>), Error>
where
    T: SqlQuery + SqlParams,
    F: FnMut(&Row) -> Result<R, Error>,
{
    let result = tx.select_all(entity, to_model)?;
    Ok((tx, result))
}

// Geriye dönük uyumluluk için eski tx_get fonksiyonunu koruyalım
#[deprecated(
    since = "0.2.0",
    note = "Renamed to `tx_fetch`. Please use `tx_fetch` function instead."
)]
/// # tx_get
/// 
/// Transaction içinde tek bir kaydı getirir.
/// 
/// This function is deprecated. Please use `tx_fetch` instead.
pub fn tx_get<'a, T>(mut tx: Transaction<'a>, entity: &T) -> Result<(Transaction<'a>, T), Error>
where
    T: SqlQuery + FromRow + SqlParams,
{
    let result = tx.fetch(entity)?;
    Ok((tx, result))
}

// Geriye dönük uyumluluk için eski tx_get_all fonksiyonunu koruyalım
#[deprecated(
    since = "0.2.0",
    note = "Renamed to `tx_fetch_all`. Please use `tx_fetch_all` function instead."
)]
/// # tx_get_all
/// 
/// Transaction içinde birden fazla kaydı getirir.
/// 
/// This function is deprecated. Please use `tx_fetch_all` instead.
pub fn tx_get_all<'a, T>(mut tx: Transaction<'a>, entity: &T) -> Result<(Transaction<'a>, Vec<T>), Error>
where
    T: SqlQuery + FromRow + SqlParams,
{
    let result = tx.fetch_all(entity)?;
    Ok((tx, result))
}
