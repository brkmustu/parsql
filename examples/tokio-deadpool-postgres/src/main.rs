mod config;
mod models;
mod repository;

use crate::config::DatabaseConfig;
use crate::models::{UserInsert, UserUpdate, UserDelete, UserById, UsersByState, UserStatusQuery};
use crate::repository::UserRepository;
use dotenv::dotenv;
use parsql::deadpool_postgres::Error;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // .env dosyasını yükle (varsa)
    dotenv().ok();
    
    // Bir zaman damgası oluştur - benzersiz e-posta adresleri için
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Zaman hesaplanamadı")
        .as_secs();
    
    // Veritabanı tablolarını oluşturmak için SQL
    const DROP_TABLE_SQL: &str = "DROP TABLE IF EXISTS users";
    const CREATE_TABLE_SQL: &str = r#"
    CREATE TABLE users (
        id BIGSERIAL PRIMARY KEY,
        name TEXT NOT NULL,
        email TEXT NOT NULL,
        state SMALLINT NOT NULL
    )
    "#;
    
    // Tabloyu temizle
    const TRUNCATE_TABLE_SQL: &str = "TRUNCATE TABLE users RESTART IDENTITY CASCADE";
    
    // Veritabanı konfigürasyonu
    let db_config = DatabaseConfig::from_env();
    println!("Veritabanına bağlanılıyor: {}:{}/{}", db_config.host, db_config.port, db_config.dbname);
    
    // Veritabanı havuzu oluştur
    let pool = db_config.create_pool();
    
    // Veritabanı tablosunu oluştur
    let client = pool.get().await.expect("Havuzdan client alınamadı");
    // Önce tabloyu temizle
    client.execute(DROP_TABLE_SQL, &[]).await?;
    println!("Varsa mevcut tablo silindi");
    // Sonra yeni tabloyu oluştur
    client.execute(CREATE_TABLE_SQL, &[]).await?;
    println!("Veritabanı tablosu oluşturuldu");
    
    // Tabloyu temizle
    client.execute(TRUNCATE_TABLE_SQL, &[]).await?;
    println!("Veritabanı tablosu temizlendi");
    
    // Repository oluştur
    let user_repo = UserRepository::new(pool);
    
    // CRUD işlemlerini göster
    demo_crud_operations(&user_repo, timestamp).await?;
    
    println!("İşlem başarıyla tamamlandı!");
    Ok(())
}

async fn demo_crud_operations(repo: &UserRepository, timestamp: u64) -> Result<(), Error> {
    println!("\n=== CRUD İşlemleri Demosu ===");
    
    // 1. Kullanıcı ekleme
    println!("\n1. Kullanıcı Ekleme");
    let new_user = UserInsert::new(
        "Mehmet Yılmaz", 
        &format!("mehmet.yilmaz-{}@example.com", timestamp),
        1 // Aktif kullanıcı (state=1)
    );
    let user_id = repo.insert_user(new_user).await?;
    println!("Kullanıcı eklendi, ID: {}", user_id);
    
    // 2. Kullanıcı güncelleme
    println!("\n2. Kullanıcı Güncelleme");
    let user = repo.get_users_by_state(1).await?;
    if let Some(first_user) = user.first() {
        let user_id = first_user.id;
        let update_user = models::UserUpdate::new(
            user_id,
            "Mehmet Yılmaz (Güncellendi)",
            &format!("mehmet.updated-{}@example.com", timestamp)
        );
        let updated = repo.update_user(update_user).await?;
        println!("Kullanıcı güncellendi: {}", updated);
        
        // Güncellenen kullanıcıyı getir
        let updated_user = repo.get_user_by_id(user_id).await?;
        println!("Güncellenmiş kullanıcı: {:?}", updated_user);
    } else {
        println!("Güncellenecek kullanıcı bulunamadı");
    }
    
    // 3. Özel sorgu ile kullanıcıları getirme (durum bilgisi ile)
    println!("\n3. Özel Sorgu ile Kullanıcıları Getirme");
    let users = repo.get_users_with_status(1).await?;
    println!("Kullanıcı durumları:");
    for user in &users {
        println!("  - {}", user);
    }
    
    // 4. Transaction ile kullanıcı ekleme
    println!("\n4. Transaction ile Kullanıcı Ekleme");
    let tx_user = UserInsert::new(
        "Ali Veli", 
        &format!("ali.veli-{}@example.com", timestamp),
        1 // Aktif kullanıcı (state=1)
    );
    let tx_result = repo.create_user_with_transaction(tx_user).await?;
    println!("Transaction ile kullanıcı eklendi, kullanıcı ID: {}", tx_result);
    
    // 5. Tüm aktif kullanıcıları listeleme
    println!("\n5. Tüm Aktif Kullanıcıları Listeleme");
    let all_active = repo.get_users_by_state(1).await?;
    println!("Toplam {} aktif kullanıcı bulundu:", all_active.len());
    for user in &all_active {
        println!("  - ID: {}, Ad: {}, E-posta: {}", user.id, user.name, user.email);
    }
    
    // 6. (Opsiyonel) Bir kullanıcıyı silme
    if all_active.len() > 1 {
        println!("\n6. Kullanıcı Silme");
        let user_to_delete = all_active.last().unwrap().id;
        let deleted = repo.delete_user(user_to_delete).await?;
        println!("Kullanıcı silindi (ID: {}), etkilenen satır sayısı: {}", user_to_delete, deleted);
    }
    
    Ok(())
}

// Demo projesi olduğu için ana veritabanı şemasını oluşturacak yardımcı fonksiyonlar
// Gerçek uygulamalarda migration kullanımı tercih edilmelidir 