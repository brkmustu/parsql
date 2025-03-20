use deadpool_postgres::Pool;
use parsql::deadpool_postgres::{delete, get, get_all, insert, select_all, update, Error};
use tokio_postgres::Row as PgRow;

use crate::models::{UserById, UserDelete, UserInsert, UserUpdate, UsersByState, UserStatusQuery};

// Repository yapısı - Veritabanı işlemleri için
pub struct UserRepository {
    pool: Pool,
}

impl UserRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    // Kullanıcı ekleme
    pub async fn insert_user(&self, user: UserInsert) -> Result<i64, Error> {
        // Parsql'in insert fonksiyonu, doğrudan havuzla çalışır
        let result = insert(&self.pool, user).await?;
        Ok(result as i64) // Eklenen satır sayısı
    }

    // Kullanıcı güncelleme
    pub async fn update_user(&self, user: UserUpdate) -> Result<bool, Error> {
        // Parsql'in update fonksiyonu, doğrudan havuzla çalışır
        update(&self.pool, user).await
    }

    // Kullanıcı silme
    pub async fn delete_user(&self, id: i64) -> Result<u64, Error> {
        // Parsql'in delete fonksiyonu, doğrudan havuzla çalışır
        let user_delete = UserDelete::new(id);
        delete(&self.pool, user_delete).await
    }

    // ID'ye göre kullanıcı getirme
    pub async fn get_user_by_id(&self, id: i64) -> Result<UserById, Error> {
        // Parsql'in get fonksiyonu, doğrudan havuzla çalışır
        let user_query = UserById::new(id);
        get(&self.pool, &user_query).await
    }

    // State durumuna göre kullanıcıları getirme
    pub async fn get_users_by_state(&self, state: i16) -> Result<Vec<UsersByState>, Error> {
        // Parsql'in get_all fonksiyonu, doğrudan havuzla çalışır
        let query = UsersByState::new(state);
        get_all(&self.pool, &query).await
    }

    // Özel sorgu ile kullanıcıları getirme (durum bilgisi ile)
    pub async fn get_users_with_status(&self, state: i16) -> Result<Vec<UserWithStatus>, Error> {
        // Parsql'in select_all fonksiyonu ile özel dönüşüm yapma
        let query = UserStatusQuery::new(state);
        
        // Satırdan özet modeline dönüştürme fonksiyonu
        let row_to_status = |row: &PgRow| UserWithStatus {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            status: row.get("status"),
        };
        
        select_all(&self.pool, query, row_to_status).await
    }

    // Transaction kullanarak birden fazla işlemi atomik olarak gerçekleştirme
    pub async fn create_user_with_transaction(&self, user: UserInsert) -> Result<i64, Error> {
        // Havuzdan client al
        let mut client = self.pool.get().await.map_err(|e| {
            // PoolError'ı hata mesajına dönüştür
            eprintln!("Pool hatası: {}", e);
            Error::__private_api_timeout()
        })?;
        
        // Transaction başlat
        let tx = client.transaction().await?;
        
        // Transaction ile insert
        let sql = "INSERT INTO users (name, email, state) VALUES ($1, $2, $3) RETURNING id";
        let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
            &user.name,
            &user.email,
            &user.state,
        ];
        
        let row = tx.query_one(sql, params).await?;
        let user_id: i64 = row.get(0);
        
        // Başarılıysa commit et
        tx.commit().await?;
        
        Ok(user_id)
    }
}

// Kullanıcı durum bilgisi ile model - Özel dönüşüm için
#[derive(Debug, Clone)]
pub struct UserWithStatus {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub status: String,
}

// UserWithStatus için Display trait implementasyonu
impl std::fmt::Display for UserWithStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID: {}, Ad: {}, Durum: {}", self.id, self.name, self.status)
    }
} 