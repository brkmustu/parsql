use deadpool_postgres::Pool;
use tokio_postgres::{Error, Row};
use crate::{SqlQuery, SqlParams, UpdateParams, FromRow};

// Daha basit bir yaklaşım: PoolError'dan genel bir Error oluştur
fn pool_err_to_io_err(e: deadpool_postgres::PoolError) -> Error {
    // Bu özel fonksiyon tokio_postgres'in sağladığı timeout hatasını döndürür
    // Güzel bir çözüm değil, ama çalışır bir örnek için kullanılabilir
    let err = Error::__private_api_timeout();
    
    // Debug süreci için stderr'e hatayı yazdıralım
    eprintln!("Pool bağlantı hatası: {}", e);
    
    err
}

/// CrudOps trait'i, Pool nesnesi için CRUD işlemlerini extension method olarak sağlar.
/// Bu trait, Pool üzerinde doğrudan CRUD işlemlerini çağırmayı mümkün kılar.
#[async_trait::async_trait]
pub trait CrudOps {
    /// Veritabanına yeni bir kayıt ekler.
    /// 
    /// # Parametreler
    /// * `entity` - Eklenecek veri nesnesi (SqlQuery ve SqlParams trait'lerini uygulamalıdır)
    /// 
    /// # Dönüş Değeri
    /// * `Result<u64, Error>` - Eklenen kayıt sayısı veya hata
    async fn insert<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlQuery + SqlParams + Send + Sync;
    
    /// Veritabanındaki mevcut bir kaydı günceller.
    /// 
    /// # Parametreler
    /// * `entity` - Güncelleme bilgilerini içeren veri nesnesi (SqlQuery ve UpdateParams trait'lerini uygulamalıdır)
    /// 
    /// # Dönüş Değeri
    /// * `Result<u64, Error>` - Güncellenen kayıt sayısı veya hata
    async fn update<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlQuery + UpdateParams + Send + Sync;
    
    /// Veritabanından bir kaydı siler.
    /// 
    /// # Parametreler
    /// * `entity` - Silinecek kaydın bilgilerini içeren veri nesnesi (SqlQuery ve SqlParams trait'lerini uygulamalıdır)
    /// 
    /// # Dönüş Değeri
    /// * `Result<u64, Error>` - Silinen kayıt sayısı veya hata
    async fn delete<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlQuery + SqlParams + Send + Sync;
    
    /// Belirtilen kriterlere uygun tek bir kaydı getirir.
    /// 
    /// # Parametreler
    /// * `params` - Sorgu parametreleri (SqlQuery, FromRow ve SqlParams trait'lerini uygulamalıdır)
    /// 
    /// # Dönüş Değeri
    /// * `Result<T, Error>` - Getirilen kayıt veya hata
    async fn fetch<T>(&self, params: &T) -> Result<T, Error>
    where
        T: SqlQuery + FromRow + SqlParams + Send + Sync;
    
    /// Belirtilen kriterlere uygun tüm kayıtları getirir.
    /// 
    /// # Parametreler
    /// * `params` - Sorgu parametreleri (SqlQuery, FromRow ve SqlParams trait'lerini uygulamalıdır)
    /// 
    /// # Dönüş Değeri
    /// * `Result<Vec<T>, Error>` - Getirilen kayıtlar veya hata
    async fn fetch_all<T>(&self, params: &T) -> Result<Vec<T>, Error>
    where
        T: SqlQuery + FromRow + SqlParams + Send + Sync;
    
    /// Belirtilen özel dönüşüm fonksiyonunu kullanarak tek bir kaydı getirir.
    /// 
    /// # Parametreler
    /// * `entity` - Sorgu parametreleri (SqlQuery ve SqlParams trait'lerini uygulamalıdır)
    /// * `to_model` - Row -> R dönüşümünü gerçekleştiren fonksiyon
    /// 
    /// # Dönüş Değeri
    /// * `Result<R, Error>` - Dönüştürülen kayıt veya hata
    async fn select<T, R, F>(&self, entity: T, to_model: F) -> Result<R, Error>
    where
        T: SqlQuery + SqlParams + Send + Sync,
        F: FnOnce(&Row) -> Result<R, Error> + Send + Sync;
    
    /// Belirtilen özel dönüşüm fonksiyonunu kullanarak tüm kayıtları getirir.
    /// 
    /// # Parametreler
    /// * `entity` - Sorgu parametreleri (SqlQuery ve SqlParams trait'lerini uygulamalıdır)
    /// * `to_model` - Row -> R dönüşümünü gerçekleştiren fonksiyon
    /// 
    /// # Dönüş Değeri
    /// * `Result<Vec<R>, Error>` - Dönüştürülen kayıtlar veya hata
    async fn select_all<T, R, F>(&self, entity: T, to_model: F) -> Result<Vec<R>, Error>
    where
        T: SqlQuery + SqlParams + Send + Sync,
        F: Fn(&Row) -> R + Send + Sync;
}

/// Pool nesnesi için CrudOps trait'inin implementasyonu
#[async_trait::async_trait]
impl CrudOps for Pool {
    async fn insert<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlQuery + SqlParams + Send + Sync
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;
        let sql = T::query();
        
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let params = entity.params();
        client.execute(&sql, &params).await
    }

    async fn update<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlQuery + UpdateParams + Send + Sync
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;
        let sql = T::query();
        
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let params = entity.params();
        client.execute(&sql, &params).await
    }

    async fn delete<T>(&self, entity: T) -> Result<u64, Error>
    where
        T: SqlQuery + SqlParams + Send + Sync
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;
        let sql = T::query();
        
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let params = entity.params();
        client.execute(&sql, &params).await
    }

    async fn fetch<T>(&self, params: &T) -> Result<T, Error>
    where
        T: SqlQuery + FromRow + SqlParams + Send + Sync
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;
        let sql = T::query();
        
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let query_params = params.params();
        let row = client.query_one(&sql, &query_params).await?;
        T::from_row(&row)
    }

    async fn fetch_all<T>(&self, params: &T) -> Result<Vec<T>, Error>
    where
        T: SqlQuery + FromRow + SqlParams + Send + Sync
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;
        let sql = T::query();
        
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let query_params = params.params();
        let rows = client.query(&sql, &query_params).await?;
        
        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(T::from_row(&row)?);
        }
        
        Ok(results)
    }

    async fn select<T, R, F>(&self, entity: T, to_model: F) -> Result<R, Error>
    where
        T: SqlQuery + SqlParams + Send + Sync,
        F: FnOnce(&Row) -> Result<R, Error> + Send + Sync
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;
        let sql = T::query();
        
        if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
            println!("[PARSQL-TOKIO-POSTGRES-POOL] Execute SQL: {}", sql);
        }

        let params = entity.params();
        let row = client.query_one(&sql, &params).await?;
        to_model(&row)
    }

    async fn select_all<T, R, F>(&self, entity: T, to_model: F) -> Result<Vec<R>, Error>
    where
        T: SqlQuery + SqlParams + Send + Sync,
        F: Fn(&Row) -> R + Send + Sync
    {
        let client = self.get().await.map_err(pool_err_to_io_err)?;
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
} 