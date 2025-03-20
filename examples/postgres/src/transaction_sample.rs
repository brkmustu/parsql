use parsql::macros::{Insertable, SqlParams, Queryable, FromRow, Updateable, UpdateParams, Deletable};
use parsql::postgres::{self, CrudOps, SqlParams, UpdateParams, SqlQuery, FromRow};
use parsql::postgres::transactional::{begin, tx_insert, tx_update, tx_get, tx_delete};
use postgres::{Client, Error, ToSql, Row};

// Kullanıcı ekleme için veri yapısı
#[derive(Insertable, SqlParams)]
#[table("users")]
pub struct TxInsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}

// Post ekleme için veri yapısı
#[derive(Insertable, SqlParams)]
#[table("posts")]
pub struct TxInsertPost {
    pub user_id: i32,
    pub content: String,
    pub state: i16,
}

// Kullanıcı güncelleme için veri yapısı
#[derive(Updateable, UpdateParams)]
#[table("users")]
#[update("name, email, state")]
#[where_clause("id = $")]
pub struct TxUpdateUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub state: i16,
}

// Kullanıcı sorgulama için veri yapısı
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("id = $")]
pub struct TxGetUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub state: i16,
}

// Kullanıcı silme için veri yapısı
#[derive(Deletable, SqlParams)]
#[table("users")]
#[where_clause("id = $")]
pub struct TxDeleteUser {
    pub id: i32,
}

/// Örnek 1: CrudOps trait'ini doğrudan Transaction üzerinde kullanma
pub fn transaction_with_crud_ops(client: &mut Client) -> Result<(), Error> {
    println!("\n--- Transaction Örneği 1: CrudOps trait'ini kullanarak ---");
    
    // Transaction başlat
    let mut tx = client.transaction()?;
    
    // Kullanıcı ekle
    let insert_user = TxInsertUser {
        name: "Transaction User 1".to_string(),
        email: "txuser1@example.com".to_string(),
        state: 1,
    };
    
    let rows_affected = tx.insert(insert_user)?;
    println!("Kullanıcı eklendi, etkilenen satır: {}", rows_affected);
    
    // Kullanıcı ID'sini al
    let rows = tx.query("SELECT lastval()", &[])?;
    let user_id: i64 = rows[0].get(0);
    let user_id = user_id as i32; // i64'ten i32'ye dönüştür
    println!("Eklenen kullanıcı ID: {}", user_id);
    
    // Post ekle
    let insert_post = TxInsertPost {
        user_id,
        content: "Transaction Örnek Post 1".to_string(),
        state: 1,
    };
    
    let rows_affected = tx.insert(insert_post)?;
    println!("Post eklendi, etkilenen satır: {}", rows_affected);
    
    // Kullanıcıyı güncelle
    let update_user = TxUpdateUser {
        id: user_id,
        name: "Updated Transaction User 1".to_string(),
        email: "updated.txuser1@example.com".to_string(),
        state: 2,
    };
    
    let rows_affected = tx.update(update_user)?;
    println!("Kullanıcı güncellendi, etkilenen satır: {}", rows_affected);
    
    // Kullanıcıyı getir
    let get_user = TxGetUser {
        id: user_id,
        name: String::new(),
        email: String::new(),
        state: 0,
    };
    
    let user = tx.fetch(&get_user)?;
    println!("Güncellenmiş kullanıcı: {:?}", user);
    
    // Transaction'ı commit et
    tx.commit()?;
    println!("Transaction başarıyla tamamlandı.");
    
    Ok(())
}

/// Örnek 2: Transaction yardımcı fonksiyonlarını kullanma
pub fn transaction_with_helper_functions(client: &mut Client) -> Result<(), Error> {
    println!("\n--- Transaction Örneği 2: Yardımcı fonksiyonları kullanarak ---");
    
    // Transaction başlat
    let mut tx = begin(client)?;
    println!("Transaction başlatıldı");
    
    // Kullanıcı ekle
    let insert_user = TxInsertUser {
        name: "Transaction User 2".to_string(),
        email: "txuser2@example.com".to_string(),
        state: 1,
    };
    
    let (mut tx, rows_affected) = tx_insert(tx, insert_user)?;
    println!("Kullanıcı eklendi, etkilenen satır: {}", rows_affected);
    
    // Kullanıcı ID'sini al
    let rows = tx.query("SELECT lastval()", &[])?;
    let user_id: i64 = rows[0].get(0);
    let user_id = user_id as i32; // i64'ten i32'ye dönüştür
    println!("Eklenen kullanıcı ID: {}", user_id);
    
    // Post ekle
    let insert_post = TxInsertPost {
        user_id,
        content: "Transaction Örnek Post 2".to_string(),
        state: 1,
    };
    
    let (mut tx, rows_affected) = tx_insert(tx, insert_post)?;
    println!("Post eklendi, etkilenen satır: {}", rows_affected);
    
    // Kullanıcıyı güncelle
    let update_user = TxUpdateUser {
        id: user_id,
        name: "Updated Transaction User 2".to_string(),
        email: "updated.txuser2@example.com".to_string(),
        state: 2,
    };
    
    let (mut tx, rows_affected) = tx_update(tx, update_user)?;
    println!("Kullanıcı güncellendi, etkilenen satır: {}", rows_affected);
    
    // Kullanıcıyı getir
    let get_user = TxGetUser {
        id: user_id,
        name: String::new(),
        email: String::new(),
        state: 0,
    };
    
    let (tx, user) = tx_get(tx, &get_user)?;
    println!("Güncellenmiş kullanıcı: {:?}", user);
    
    // Transaction'ı commit et
    tx.commit()?;
    println!("Transaction başarıyla tamamlandı.");
    
    Ok(())
}

/// Örnek 3: Transaction hata durumunu gösterme (rollback)
pub fn transaction_with_rollback(client: &mut Client) -> Result<(), Error> {
    println!("\n--- Transaction Örneği 3: Hata durumunda rollback ---");
    
    // Mevcut kullanıcı sayısını kontrol et
    let count_before = client.query_one("SELECT COUNT(*) FROM users", &[])?;
    let count_before: i64 = count_before.get(0);
    println!("İşlem öncesi kullanıcı sayısı: {}", count_before);
    
    // Transaction başlat
    let tx_result = {
        let mut tx = begin(client)?;
        println!("Transaction başlatıldı");
        
        // Kullanıcı ekle
        let insert_user = TxInsertUser {
            name: "Transaction User 3".to_string(),
            email: "txuser3@example.com".to_string(),
            state: 1,
        };
        
        let (mut tx, rows_affected) = tx_insert(tx, insert_user)?;
        println!("Kullanıcı eklendi, etkilenen satır: {}", rows_affected);
        
        // Kullanıcı ID'sini al
        let rows = tx.query("SELECT lastval()", &[])?;
        let user_id: i64 = rows[0].get(0);
        let user_id = user_id as i32; // i64'ten i32'ye dönüştür
        println!("Eklenen kullanıcı ID: {}", user_id);
        
        // Bilerek hata oluşturan bir sorgu çalıştır
        println!("Hata oluşturacak sorgu çalıştırılıyor...");
        let result = tx.execute("INSERT INTO nonexistent_table (column) VALUES ('value')", &[]);
        
        if let Err(e) = result {
            println!("Beklenen hata oluştu: {}", e);
            println!("Transaction otomatik olarak rollback olacak");
            // tx scope'un sonunda drop edilecek, rollback olacak
            Err(e)
        } else {
            // Hata beklendiği için bu bölümün çalışmaması gerekir
            println!("Beklenmeyen durum: Hata oluşmadı!");
            tx.commit()?;
            Ok(())
        }
    };
    
    // tx artık burada düşürüldü (drop), dolayısıyla client tekrar kullanılabilir
    
    // Hata oluşmasını bekliyoruz, ama programın çökmesini istemiyoruz
    let _ = tx_result; // Hata burada yutuldu
    
    // Transaction sonrası kullanıcı sayısını kontrol et
    // Rollback olduğu için sayı değişmemiş olmalı
    let count_after = client.query_one("SELECT COUNT(*) FROM users", &[])?;
    let count_after: i64 = count_after.get(0);
    println!("İşlem sonrası kullanıcı sayısı: {}", count_after);
    println!("Değişim: {}", count_after - count_before);
    
    Ok(())
}

/// Örnek 4: Karmaşık transaction senaryosu (birden fazla işlem)
pub fn complex_transaction_example(client: &mut Client) -> Result<(), Error> {
    println!("\n--- Transaction Örneği 4: Karmaşık transaction senaryosu ---");
    
    // Transaction başlat
    let mut tx = client.transaction()?;
    
    // 1. Yeni bir kullanıcı ekle
    let insert_user = TxInsertUser {
        name: "Complex User".to_string(),
        email: "complex@example.com".to_string(),
        state: 1,
    };
    
    let rows_affected = tx.insert(insert_user)?;
    println!("Kullanıcı eklendi, etkilenen satır: {}", rows_affected);
    
    // Kullanıcı ID'sini al
    let rows = tx.query("SELECT lastval()", &[])?;
    let user_id: i64 = rows[0].get(0);
    let user_id = user_id as i32; // i64'ten i32'ye dönüştür
    println!("Eklenen kullanıcı ID: {}", user_id);
    
    // 2. Bu kullanıcı için üç post ekle
    for i in 1..=3 {
        let insert_post = TxInsertPost {
            user_id,
            content: format!("Complex Post {}", i),
            state: 1,
        };
        
        let rows_affected = tx.insert(insert_post)?;
        println!("Post {} eklendi, etkilenen satır: {}", i, rows_affected);
    }
    
    // 3. Admin kullanıcısını güncelle (ID=1 olan kullanıcı)
    let admin_update = TxUpdateUser {
        id: 1,
        name: "Super Admin".to_string(),
        email: "admin@example.com".to_string(),
        state: 1,
    };
    
    let admin_updated = tx.update(admin_update)?;
    println!("Admin kullanıcısı güncellendi, etkilenen satır: {}", admin_updated);
    
    // 4. Kullanıcı durumu 2 olan kullanıcıları 3 olarak güncelle
    let state_updated = tx.execute(
        "UPDATE users SET state = 3 WHERE state = 2", 
        &[]
    )?;
    println!("Durum güncellemesi, etkilenen satır: {}", state_updated);
    
    // Transaction'ı commit et
    tx.commit()?;
    println!("Karmaşık transaction başarıyla tamamlandı.");
    
    Ok(())
}

/// Örnek 5: Transaction'da silme işlemi
pub fn transaction_with_delete(client: &mut Client) -> Result<(), Error> {
    println!("\n--- Transaction Örneği 5: Silme işlemi ---");
    
    // Önce ekleyeceğimiz kullanıcı için bir transaction başlat
    let mut tx = begin(client)?;
    
    // Kullanıcı ekle
    let insert_user = TxInsertUser {
        name: "Temporary User".to_string(),
        email: "temp@example.com".to_string(),
        state: 1,
    };
    
    let (mut tx, _) = tx_insert(tx, insert_user)?;
    
    // ID'yi al
    let rows = tx.query("SELECT lastval()", &[])?;
    let user_id: i64 = rows[0].get(0);
    let user_id = user_id as i32; // i64'ten i32'ye dönüştür
    println!("Geçici kullanıcı ID: {}", user_id);
    
    // Bu kullanıcıyı eklerken bir post da ekle
    let insert_post = TxInsertPost {
        user_id,
        content: "Temporary Post".to_string(),
        state: 1,
    };
    
    let (tx, _) = tx_insert(tx, insert_post)?;
    
    // Transaction'ı commit et
    tx.commit()?;
    println!("Geçici kullanıcı ve post eklendi");
    
    // Şimdi bu kullanıcıyı ve postlarını silmek için yeni bir transaction başlat
    let mut tx = begin(client)?;
    
    // Önce postları sil (foreign key kısıtlaması var)
    let posts_deleted = tx.execute(
        "DELETE FROM posts WHERE user_id = $1", 
        &[&user_id]
    )?;
    println!("Silinen post sayısı: {}", posts_deleted);
    
    // Şimdi kullanıcıyı sil
    let delete_user = TxDeleteUser { id: user_id };
    
    let (tx, rows_affected) = tx_delete(tx, delete_user)?;
    println!("Kullanıcı silindi, etkilenen satır: {}", rows_affected);
    
    // Transaction'ı commit et
    tx.commit()?;
    println!("Silme işlemi başarıyla tamamlandı.");
    
    Ok(())
}
