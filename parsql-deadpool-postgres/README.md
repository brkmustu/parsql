# parsql-deadpool-postgres

Parsql için Deadpool PostgreSQL entegrasyon küfesidir. Bu paket, parsql'in deadpool-postgres kütüphanesi ile asenkron bağlantı havuzu yönetimini destekleyen API'leri içerir.

## Özellikler

- Deadpool ile PostgreSQL bağlantı havuzu yönetimi
- Asenkron PostgreSQL işlemleri (tokio runtime ile)
- Otomatik SQL sorgu oluşturma
- Güvenli parametre yönetimi
- Generic CRUD işlemleri (get, insert, update, delete)
- Pool nesnesi için extension method'lar (doğrudan pool üzerinden CRUD işlemleri)
- Veritabanı satırlarını struct'lara dönüştürme
- Özel satır dönüşümleri
- SQL Injection saldırılarına karşı otomatik koruma
- Transaction desteği

## Güvenlik Özellikleri

### SQL Injection Koruması

parsql-deadpool-postgres, SQL Injection saldırılarına karşı güvenli bir şekilde tasarlanmıştır:

- Tüm kullanıcı girdileri otomatik olarak parametrize edilir
- PostgreSQL'in "$1, $2, ..." parametrelendirme yapısı otomatik olarak kullanılır
- Makrolar, SQL parametrelerini güvenli bir şekilde işleyerek injection saldırılarına karşı koruma sağlar
- Parametrelerin doğru sırada ve tipte gönderilmesi otomatik olarak yönetilir
- `#[where_clause]` ve diğer SQL bileşenlerinde kullanıcı girdileri her zaman parametrize edilir
- Bağlantı havuzu kullanırken bile güvenlik önlemleri tam olarak korunur

```rust
// SQL injection koruması örneği
#[derive(Queryable)]
#[table("users")]
#[where_clause("username = $ AND status = $")]
struct UserQuery {
    username: String,
    status: i32,
}

// Kullanıcı girdisi (potansiyel olarak zararlı) güvenle kullanılır
let query = UserQuery {
    username: kullanici_girdisi, // Bu değer direkt SQL'e eklenmez, parametrize edilir
    status: 1,
};

// Oluşturulan sorgu: "SELECT * FROM users WHERE username = $1 AND status = $2"
// Parametreler güvenli bir şekilde: [kullanici_girdisi, 1] olarak gönderilir
let user = get(&pool, &query).await?;
```

## Kurulum

Cargo.toml dosyanıza şu şekilde ekleyin:

```toml
[dependencies]
# Deadpool PostgreSQL için
parsql-deadpool-postgres = "0.3.7"
parsql-macros = "0.3.7"
deadpool-postgres = "0.14"
tokio-postgres = "0.7"
tokio = { version = "1", features = ["full"] }
```

## Temel Kullanım

Bu paket, PostgreSQL veritabanı ile çalışırken **asenkron işlemler** ve **bağlantı havuzu yönetimi** kullanır. Bu, async/await kullanımı gerektirdiği anlamına gelir.

### Bağlantı Havuzu Oluşturma

```rust
use deadpool_postgres::{Config, Runtime};
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // PostgreSQL bağlantı havuzu oluşturma
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.user = Some("postgres".to_string());
    cfg.password = Some("postgres".to_string());
    cfg.dbname = Some("test".to_string());
    
    // Bağlantı havuzunu oluşturma
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
    
    // ... havuz kullanımı ...
    
    Ok(())
}
```

### Modelleri Tanımlama

Veritabanı CRUD işlemleri için, ilgili makrolarla işaretlenmiş model struct'ları tanımlayın:

```rust
use parsql_deadpool_postgres::macros::{Insertable, Updateable, Queryable, Deletable};

// Eklemek için model
#[derive(Insertable)]
#[table("users")]
struct UserInsert {
    name: String,
    email: String,
    active: bool,
}

// Güncellemek için model
#[derive(Updateable)]
#[table("users")]
#[update("name, email, active")]
#[where_clause("id = $")]
struct UserUpdate {
    id: i32,
    name: String,
    email: String,
    active: bool,
}

// Sorgulamak için model
#[derive(Queryable)]
#[table("users")]
#[select("id, name, email, active")] 
#[where_clause("id = $")]
struct UserById {
    id: i32,
    name: String,
    email: String,
    active: bool,
}

// Silmek için model
#[derive(Deletable)]
#[table("users")]
#[where_clause("id = $")]
struct UserDelete {
    id: i32,
}
```

## CRUD İşlemleri

CRUD işlemlerini gerçekleştirmek için iki farklı yaklaşım kullanabilirsiniz:

1. Fonksiyon çağrıları ile
2. Extension method'lar ile (doğrudan Pool nesnesi üzerinden)

### Fonksiyon Çağrıları ile Kullanım

#### Veri Ekleme

```rust
use parsql_deadpool_postgres::insert;

let user = UserInsert {
    name: "Ahmet Yılmaz".to_string(),
    email: "ahmet@example.com".to_string(),
    active: true,
};

let result = insert(&pool, user).await?;
println!("Eklenen kayıt sayısı: {}", result);
```

#### Veri Güncelleme

```rust
use parsql_deadpool_postgres::update;

let user = UserUpdate {
    id: 1,
    name: "Ahmet Yılmaz (Güncellendi)".to_string(),
    email: "ahmet.updated@example.com".to_string(),
    active: true,
};

let rows_affected = update(&pool, user).await?;
println!("Güncellenen kayıt sayısı: {}", rows_affected);
```

#### Veri Sorgulama

```rust
use parsql_deadpool_postgres::{get, get_all};

// Tek bir kayıt getirme
let query = UserById { id: 1, ..Default::default() };
let user = get(&pool, &query).await?;

// Birden fazla kayıt getirme
let query = UsersByActive { active: true, ..Default::default() };
let active_users = get_all(&pool, &query).await?;
```

#### Veri Silme

```rust
use parsql_deadpool_postgres::delete;

let user_delete = UserDelete { id: 1 };
let deleted_count = delete(&pool, user_delete).await?;
println!("Silinen kayıt sayısı: {}", deleted_count);
```

### Extension Method'lar ile Kullanım

Pool nesnesi üzerinde doğrudan çalışan extension method'ları kullanmak için `CrudOps` trait'ini içe aktarın:

```rust
use parsql_deadpool_postgres::CrudOps;

// Extension method kullanarak ekleme
let user = UserInsert {
    name: "Ahmet Yılmaz".to_string(),
    email: "ahmet@example.com".to_string(),
    active: true,
};

let result = pool.insert(user).await?;
println!("Eklenen kayıt sayısı: {}", result);

// Extension method kullanarak güncelleme
let user_update = UserUpdate {
    id: 1,
    name: "Ahmet Yılmaz (Güncellendi)".to_string(),
    email: "ahmet.updated@example.com".to_string(),
    active: true,
};

let rows_affected = pool.update(user_update).await?;
println!("Güncellenen kayıt sayısı: {}", rows_affected);

// Extension method kullanarak kayıt getirme
let query = UserById { id: 1, ..Default::default() };
let user = pool.get(&query).await?;
println!("Kullanıcı: {:?}", user);

// Extension method kullanarak birden fazla kayıt getirme
let active_query = UsersByActive { active: true, ..Default::default() };
let active_users = pool.get_all(&active_query).await?;
println!("Aktif kullanıcı sayısı: {}", active_users.len());

// Extension method kullanarak silme
let user_delete = UserDelete { id: 1 };
let deleted_count = pool.delete(user_delete).await?;
println!("Silinen kayıt sayısı: {}", deleted_count);
```

## Transaction İşlemleri

Transaction işlemlerini gerçekleştirmek için iki farklı yaklaşım kullanabilirsiniz:

1. Extension method'lar ile (doğrudan Transaction nesnesi üzerinden)
2. Transaction helper fonksiyonları ile

### Transaction Extension Method'ları ile Kullanım

Transaction nesnesi üzerinde doğrudan çalışan extension method'ları kullanmak için `TransactionOps` trait'ini içe aktarın:

```rust
use parsql_deadpool_postgres::{CrudOps, TransactionOps};
use tokio_postgres::NoTls;
use deadpool_postgres::{Config, Runtime};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.dbname = Some("test".to_string());
    
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
    let client = pool.get().await?;
    
    // Transaction başlatma
    let tx = client.transaction().await?;
    
    // Transaction içinde extension method kullanarak ekleme
    let user = UserInsert {
        name: "Ahmet Yılmaz".to_string(),
        email: "ahmet@example.com".to_string(),
        active: true,
    };
    let result = tx.insert(user).await?;
    
    // Transaction içinde extension method kullanarak güncelleme
    let user_update = UserUpdate {
        id: 1,
        name: "Ahmet Yılmaz (Güncellendi)".to_string(),
        email: "ahmet.updated@example.com".to_string(),
        active: true,
    };
    let rows_affected = tx.update(user_update).await?;
    
    // İşlem başarılı olursa commit
    tx.commit().await?;
    
    Ok(())
}
```

Transaction nesnesi üzerinde şu extension methodlar kullanılabilir:
- `tx.insert(entity)` - Kayıt ekler
- `tx.update(entity)` - Kayıt günceller
- `tx.delete(entity)` - Kayıt siler
- `tx.get(params)` - Tek bir kayıt getirir
- `tx.get_all(params)` - Birden fazla kayıt getirir
- `tx.select(entity, to_model)` - Özel dönüştürücü fonksiyon ile tek kayıt getirir
- `tx.select_all(entity, to_model)` - Özel dönüştürücü fonksiyon ile çoklu kayıt getirir

### Transaction Helper Fonksiyonları ile Kullanım

Transaction helper fonksiyonlarını kullanmak için `transactional` modülünü içe aktarın:

```rust
use parsql_deadpool_postgres::transactional::{begin, tx_insert, tx_update};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.dbname = Some("test".to_string());
    
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
    let mut client = pool.get().await?;
    
    // Transaction başlatma
    let tx = begin(&mut client).await?;
    
    // Transaction içinde ekleme
    let user = UserInsert {
        name: "Ahmet Yılmaz".to_string(),
        email: "ahmet@example.com".to_string(),
        active: true,
    };
    let (tx, result) = tx_insert(tx, user).await?;
    
    // Transaction içinde güncelleme
    let user_update = UserUpdate {
        id: 1,
        name: "Ahmet Yılmaz (Güncellendi)".to_string(),
        email: "ahmet.updated@example.com".to_string(),
        active: true,
    };
    let (tx, rows_affected) = tx_update(tx, user_update).await?;
    
    // İşlem başarılı olursa commit
    tx.commit().await?;
    
    Ok(())
}
```

Transaction helper fonksiyonları şunları içerir:
- `begin(client)` - Yeni bir transaction başlatır
- `tx_insert(tx, entity)` - Transaction içinde kayıt ekler
- `tx_update(tx, entity)` - Transaction içinde kayıt günceller
- `tx_delete(tx, entity)` - Transaction içinde kayıt siler
- `tx_get(tx, params)` - Transaction içinde tek bir kayıt getirir
- `tx_get_all(tx, params)` - Transaction içinde birden fazla kayıt getirir
- `tx_select(tx, entity, to_model)` - Transaction içinde özel dönüştürücü fonksiyon ile tek kayıt getirir
- `tx_select_all(tx, entity, to_model)` - Transaction içinde özel dönüştürücü fonksiyon ile çoklu kayıt getirir

## Örnek Proje

Daha kapsamlı bir örnek için, proje içindeki `/examples/tokio-deadpool-postgres` dizinine bakabilirsiniz.

## Lisans

Bu proje MIT lisansı altında lisanslanmıştır.
