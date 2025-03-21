# parsql-tokio-postgres

Parsql için Tokio PostgreSQL entegrasyon küfesidir. Bu paket, parsql'in tokio-postgres ve deadpool-postgres kütüphaneleriyle asenkron çalışmasını sağlayan API'leri içerir.

## Özellikler

- Asenkron PostgreSQL işlemleri (tokio runtime ile)
- Otomatik SQL sorgu oluşturma
- Güvenli parametre yönetimi
- Generic CRUD işlemleri (get, insert, update, delete)
- Veritabanı satırlarını struct'lara dönüştürme
- Deadpool bağlantı havuzu desteği
- SQL Injection saldırılarına karşı otomatik koruma
- Detaylı hata raporlama
- Yüksek performanslı asenkron sorgu yürütme
- Extension metotları (CrudOps trait)

## Güvenlik Özellikleri

### SQL Injection Koruması

parsql-tokio-postgres, SQL Injection saldırılarına karşı güvenli bir şekilde tasarlanmıştır:

- Tüm kullanıcı girdileri otomatik olarak parametrize edilir
- PostgreSQL'in "$1, $2, ..." parametrelendirme yapısı otomatik olarak kullanılır
- Makrolar, SQL parametrelerini güvenli bir şekilde işleyerek injection saldırılarına karşı koruma sağlar
- Parametrelerin doğru sırada ve tipte gönderilmesi otomatik olarak yönetilir
- `#[where_clause]` ve diğer SQL bileşenlerinde kullanıcı girdileri her zaman parametrize edilir
- Asenkron bağlamlarda bile güvenlik önlemleri tam olarak korunur

```rust
// SQL injection koruması örneği
#[derive(Queryable, FromRow, SqlParams)]
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
let user = get(&client, query).await?;
```

## Kurulum

Cargo.toml dosyanıza şu şekilde ekleyin:

```toml
[dependencies]
# Tokio PostgreSQL için
parsql = { version = "0.3.7", features = ["tokio-postgres"] }

# Deadpool PostgreSQL için
parsql = { version = "0.3.7", features = ["deadpool-postgres"] }
```

veya doğrudan bu paketi kullanmak isterseniz:

```toml
[dependencies]
parsql-tokio-postgres = "0.3.2"
parsql-macros = "0.3.2"
tokio-postgres = "0.7.13"
tokio = { version = "1.41.1", features = ["full"] }

# Deadpool kullanmak isterseniz
deadpool-postgres = "0.14.1"
```

## Temel Kullanım

Bu paket, PostgreSQL veritabanı ile çalışırken **asenkron işlemler** kullanır. Bu, async/await kullanımı gerektirdiği anlamına gelir.

### Bağlantı Kurma

#### Tokio PostgreSQL ile Bağlantı

```rust
use tokio_postgres::{NoTls, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // PostgreSQL bağlantısı oluşturma
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    ).await?;
    
    // Bağlantıyı arka planda çalıştır
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Bağlantı hatası: {}", e);
        }
    });
    
    // Örnek tablo oluşturma
    client.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            state SMALLINT NOT NULL
        )",
        &[],
    ).await?;
    
    // ...
    
    Ok(())
}
```

#### Deadpool PostgreSQL ile Bağlantı Havuzu

```rust
use deadpool_postgres::{Config, Client, Pool};
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Deadpool konfigürasyonu
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.user = Some("postgres".to_string());
    cfg.password = Some("postgres".to_string());
    cfg.dbname = Some("test".to_string());
    
    // Bağlantı havuzu oluşturma
    let pool = cfg.create_pool(None, NoTls)?;
    
    // Havuzdan bağlantı alma
    let client: Client = pool.get().await?;
    
    // ...
    
    Ok(())
}
```

## CRUD İşlemleri

Veritabanı işlemlerini gerçekleştirmek için iki yaklaşım sunulmaktadır:

1. Fonksiyon tabanlı yaklaşım (`get`, `insert`, `update`, vb.)
2. Extension metot yaklaşımı (`client.get()`, `client.insert()`, vb.) - `CrudOps` trait

### Extension Metot Yaklaşımı (CrudOps Trait)

```rust
use parsql::{
    macros::{Queryable, FromRow, SqlParams},
    tokio_postgres::{CrudOps},
};

#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("id = $")]
pub struct GetUser {
    pub id: i64,
    pub name: String,
    pub email: String,
}

impl GetUser {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            name: Default::default(),
            email: Default::default(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Bağlantı hatası: {}", e);
        }
    });
    
    // Extension metot kullanımı
    let get_user = GetUser::new(1);
    let user = client.get(get_user).await?;
    
    println!("Kullanıcı: {:?}", user);
    Ok(())
}
```

### Veri Okuma (Get) İşlemi

```rust
use parsql::{
    core::Queryable,
    macros::{FromRow, Queryable, SqlParams},
    tokio_postgres::{FromRow, SqlParams, get, CrudOps},
};
use tokio_postgres::{types::ToSql, Row};

#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("id = $")]
pub struct GetUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}

impl GetUser {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            name: Default::default(),
            email: Default::default(),
            state: Default::default(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Bağlantı hatası: {}", e);
        }
    });
    
    // Kullanımı (fonksiyon yaklaşımı)
    let get_user = GetUser::new(1);
    let get_result = get(&client, get_user).await?;
    
    // veya (extension metot yaklaşımı)
    let get_user = GetUser::new(1);
    let get_result = client.get(get_user).await?;
    
    println!("Kullanıcı: {:?}", get_result);
    Ok(())
}
```

### Veri Ekleme (Insert) İşlemi

```rust
use parsql::{
    core::Insertable,
    macros::{Insertable, SqlParams},
    tokio_postgres::{SqlParams, insert, CrudOps},
};
use tokio_postgres::types::ToSql;

#[derive(Insertable, SqlParams)]
#[table("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Bağlantı hatası: {}", e);
        }
    });
    
    let insert_user = InsertUser {
        name: "Ali".to_string(),
        email: "ali@parsql.com".to_string(),
        state: 1,
    };
    
    // Fonksiyon yaklaşımı
    let insert_result = insert(&client, insert_user).await?;
    
    // veya extension metot yaklaşımı
    let insert_user = InsertUser {
        name: "Mehmet".to_string(),
        email: "mehmet@parsql.com".to_string(),
        state: 1,
    };
    let insert_result = client.insert(insert_user).await?;
    
    println!("Eklenen kayıt ID: {}", insert_result);
    
    Ok(())
}
```

### Veri Güncelleme (Update) İşlemi

```rust
use parsql::{
    core::Updateable,
    macros::{UpdateParams, Updateable},
    tokio_postgres::{UpdateParams, update, CrudOps},
};

#[derive(Updateable, UpdateParams)]
#[table("users")]
#[update("name, email")]
#[where_clause("id = $")]
pub struct UpdateUser {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Bağlantı hatası: {}", e);
        }
    });
    
    let update_user = UpdateUser {
        id: 1,
        name: "Ali Yılmaz".to_string(),
        email: "ali.yilmaz@parsql.com".to_string(),
    };
    
    // Fonksiyon yaklaşımı
    let update_result = update(&client, update_user).await?;
    
    // veya extension metot yaklaşımı
    let update_user = UpdateUser {
        id: 2,
        name: "Ayşe Kaya".to_string(),
        email: "ayse.kaya@parsql.com".to_string(),
    };
    let update_result = client.update(update_user).await?;
    
    println!("Güncelleme başarılı: {}", update_result);
    
    Ok(())
}
```

### Veri Silme (Delete) İşlemi

```rust
use parsql::{
    core::Deletable,
    macros::{Deletable, SqlParams},
    tokio_postgres::{SqlParams, delete, CrudOps},
};

#[derive(Deletable, SqlParams)]
#[table("users")]
#[where_clause("id = $")]
pub struct DeleteUser {
    pub id: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Bağlantı hatası: {}", e);
        }
    });
    
    let delete_user = DeleteUser { id: 1 };
    
    // Fonksiyon yaklaşımı
    let delete_result = delete(&client, delete_user).await?;
    
    // veya extension metot yaklaşımı
    let delete_user = DeleteUser { id: 2 };
    let delete_result = client.delete(delete_user).await?;
    
    println!("Silinen kayıt sayısı: {}", delete_result);
    
    Ok(())
}
```

## Özel Sorgular

Bazen standart CRUD işlemleri yetersiz kalabilir. Özel sorguları kolayca çalıştırmak için `select` ve `select_all` işlevleri sağlanmıştır. Bunlar da hem fonksiyon hem de extension metot olarak sunulmaktadır:

```rust
use parsql::{
    core::Queryable,
    macros::{Queryable, SqlParams},
    tokio_postgres::{SqlParams, select, select_all, FromRow, CrudOps},
};
use tokio_postgres::Row;

#[derive(Queryable, SqlParams)]
#[table("users")]
#[select("SELECT u.*, p.role FROM users u JOIN profiles p ON u.id = p.user_id")]
#[where_clause("u.state = $")]
pub struct UserWithRole {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
    pub role: String,
}

// FromRow trait'ini manuel olarak uygulama
impl FromRow for UserWithRole {
    fn from_row(row: &Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            email: row.try_get("email")?,
            state: row.try_get("state")?,
            role: row.try_get("role")?,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Bağlantı hatası: {}", e);
        }
    });
    
    let query = UserWithRole {
        id: 0,
        name: String::new(),
        email: String::new(),
        state: 1, // Aktif kullanıcılar
        role: String::new(),
    };
    
    // Fonksiyon yaklaşımı - Tek bir sonuç almak için
    let user = select(&client, query.clone(), |row| UserWithRole::from_row(row)).await?;
    
    // Extension metot yaklaşımı - Tek bir sonuç almak için
    let user = client.select(query.clone(), |row| UserWithRole::from_row(row)).await?;
    
    println!("Kullanıcı: {:?}", user);
    
    // Fonksiyon yaklaşımı - Tüm sonuçları almak için
    let users = select_all(&client, query.clone(), |row| {
        UserWithRole::from_row(row).unwrap()
    }).await?;
    
    // Extension metot yaklaşımı - Tüm sonuçları almak için
    let users = client.select_all(query, |row| {
        UserWithRole::from_row(row).unwrap()
    }).await?;
    
    println!("Kullanıcı sayısı: {}", users.len());
    
    Ok(())
}
```

## Deadpool Bağlantı Havuzu ile Kullanım

Deadpool bağlantı havuzu, çok sayıda eşzamanlı veritabanı işlemi için bağlantıları etkin şekilde yönetmenizi sağlar. Bunu kullanmak için `deadpool-postgres` özelliğini etkinleştirin:

```rust
use parsql::{
    tokio_postgres::{get, FromRow, SqlParams},
    macros::{FromRow, Queryable, SqlParams},
};
use deadpool_postgres::{Config, Client, Pool};
use tokio_postgres::NoTls;

#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("state = $")]
pub struct ActiveUsers {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}

impl ActiveUsers {
    pub fn new() -> Self {
        Self {
            id: 0,
            name: String::new(),
            email: String::new(),
            state: 1, // Aktif kullanıcılar
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Deadpool konfigürasyonu
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.user = Some("postgres".to_string());
    cfg.password = Some("postgres".to_string());
    cfg.dbname = Some("test".to_string());
    
    // Bağlantı havuzu oluşturma
    let pool = cfg.create_pool(None, NoTls)?;
    
    // Havuzdan bağlantı alma
    let client: Client = pool.get().await?;
    
    // Sorgu oluşturma
    let query = ActiveUsers::new();
    
    // Aktif kullanıcıları getirme
    let active_users = get(&client, query).await?;
    println!("Aktif kullanıcı: {:?}", active_users);
    
    Ok(())
}
```

## Gelişmiş Özellikler ve Optimizasyonlar

### SQL İzleme

Hata ayıklama amacıyla, çalıştırılan SQL sorgularını izlemek için `PARSQL_TRACE` çevre değişkenini kullanabilirsiniz:

```bash
PARSQL_TRACE=1 cargo run
```

Bu, çalıştırılan tüm SQL sorgularını konsola yazdıracaktır.

### Makro Seçenekleri

Makrolar, SQL oluşturmada esneklik sağlamak için çeşitli özellikler sunar:

#### Queryable (SELECT)

```rust
#[derive(Queryable, FromRow, SqlParams)]
#[table("users")]
#[select("SELECT * FROM users")] // İsteğe bağlı özel SQL
#[where_clause("id = $ AND state = $")] // Koşullar
#[order_by("id DESC")] // Sıralama
#[limit(10)] // Limit
#[offset(5)] // Offset
struct UserQuery {
    // ...
}
```

#### Insertable

```rust
#[derive(Insertable, SqlParams)]
#[table("users")]
#[returning("id")] // INSERT...RETURNING için
struct NewUser {
    // ...
}
```

#### Updateable

```rust
#[derive(Updateable, UpdateParams)]
#[table("users")]
#[update("name, email")] // Yalnızca belirli alanları güncelle
#[where_clause("id = $")]
struct UpdateUser {
    // ...
}
```

## Performans İpuçları

* Sorgu planı ön belleğinden yararlanmak için aynı SQL yapısına sahip sorguları tekrar kullanın
* Çok sayıda sorgu için bağlantı havuzları kullanın
* Büyük veri kümeleri için `get_all` yerine sayfalama (limit ve offset) kullanın
* Filtreleri veritabanı seviyesinde uygulayın, uygulamanızda değil

## Hata Yakalama ve İşleme

```rust
async fn handle_database() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Bağlantı hatası: {}", e);
        }
    });
    
    let result = match get(&client, user_query).await {
        Ok(user) => {
            println!("Kullanıcı bulundu: {:?}", user);
            // İşlem başarılı
            Ok(())
        },
        Err(e) => match e.code() {
            // Belirli PostgreSQL hata kodlarını işleme
            Some(code) if code == &tokio_postgres::error::SqlState::UNIQUE_VIOLATION => {
                println!("Benzersizlik ihlali: {}", e);
                Err(e.into())
            },
            Some(code) if code == &tokio_postgres::error::SqlState::FOREIGN_KEY_VIOLATION => {
                println!("Yabancı anahtar ihlali: {}", e);
                Err(e.into())
            },
            _ => {
                println!("Genel veritabanı hatası: {}", e);
                Err(e.into())
            }
        },
    };
    
    result
}
```

## Lisanslama

Bu kütüphane MIT veya Apache-2.0 lisansı altında lisanslanmıştır.

## Transaction İşlemleri

Transaction işlemleri için iki farklı yaklaşım kullanabilirsiniz:

#### 1. `CrudOps` trait üzerinden Transaction nesnesiyle çalışma

`Transaction` struct'ı için `CrudOps` trait'i implement edilmiştir, bu sayede `Client` nesnesi üzerinde kullanılan extension methodlarını doğrudan `Transaction` nesnesi üzerinde de kullanabilirsiniz:

```rust
use tokio_postgres::{NoTls, Error};
use parsql::tokio_postgres::{CrudOps, transactional};
use parsql::macros::{Insertable, Updateable, SqlParams};

#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
    state: i16,
}

#[derive(Updateable, SqlParams)]
#[table("users")]
#[where_clause("id = $")]
struct ActivateUser {
    id: i64,
    state: i16,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (mut client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Bağlantı hatası: {}", e);
        }
    });
    
    // Transaction başlat
    let transaction = client.transaction().await?;
    
    // Kullanıcı ekle - CrudOps trait methodunu kullanarak
    let user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
        state: 0, // pasif
    };
    
    // Transaction içinde doğrudan insert işlemi
    let rows = transaction.insert(user).await?;
    println!("Eklenen satır sayısı: {}", rows);
    
    // Kullanıcıyı aktifleştir - CrudOps trait methodunu kullanarak
    let activate = ActivateUser {
        id: 1, // Eklenen kullanıcının ID'si
        state: 1, // aktif
    };
    
    // Transaction içinde doğrudan update işlemi
    let updated = transaction.update(activate).await?;
    println!("Güncelleme başarılı: {}", updated);
    
    // Transaction'ı commit et
    transaction.commit().await?;
    
    Ok(())
}
```

#### 2. `transactional` modülü ile çalışma

`transactional` modülündeki yardımcı fonksiyonları kullanarak işlemlerinizi gerçekleştirebilirsiniz:

```rust
use tokio_postgres::{NoTls, Error};
use parsql::tokio_postgres::transactional;
use parsql::macros::{Insertable, Updateable, SqlParams};

#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
    state: i16,
}

#[derive(Updateable, SqlParams)]
#[table("users")]
#[where_clause("id = $")]
struct ActivateUser {
    id: i64,
    state: i16,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (mut client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres dbname=test",
        NoTls,
    ).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Bağlantı hatası: {}", e);
        }
    });
    
    // Transaction başlat
    let tx = transactional::begin(&mut client).await?;
    
    // Kullanıcı ekle
    let user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
        state: 0, // pasif
    };
    
    // Transaction içinde insert işlemi
    let (tx, rows) = transactional::tx_insert(tx, user).await?;
    println!("Eklenen satır sayısı: {}", rows);
    
    // Kullanıcıyı aktifleştir
    let activate = ActivateUser {
        id: 1, // Eklenen kullanıcının ID'si
        state: 1, // aktif
    };
    
    // Transaction içinde update işlemi
    let (tx, updated) = transactional::tx_update(tx, activate).await?;
    println!("Güncelleme başarılı: {}", updated);
    
    // Transaction'ı commit et
    tx.commit().await?;
    
    Ok(())
}
```

### Transaction Helper Fonksiyonları

`transactional` modülü aşağıdaki fonksiyonları sağlar:

- `begin(&mut client)`: Yeni bir transaction başlatır
- `tx_insert(transaction, entity)`: Transaction içinde insert işlemi yapar ve transaction ile etkilenen satır sayısını döndürür
- `tx_update(transaction, entity)`: Transaction içinde update işlemi yapar ve transaction ile güncelleme durumunu döndürür
- `tx_delete(transaction, entity)`: Transaction içinde delete işlemi yapar ve transaction ile silinen satır sayısını döndürür
- `tx_get(transaction, params)`: Transaction içinde tek bir kayıt getirir ve transaction ile kaydı döndürür
- `tx_get_all(transaction, params)`: Transaction içinde birden fazla kayıt getirir ve transaction ile kayıtları döndürür
- `tx_select(transaction, entity, to_model)`: Transaction içinde özel bir sorgu çalıştırır ve belirtilen dönüşüm fonksiyonunu kullanarak sonuçları dönüştürür
- `tx_select_all(transaction, entity, to_model)`: Transaction içinde özel bir sorgu çalıştırır ve tüm sonuçları belirtilen dönüşüm fonksiyonunu kullanarak dönüştürür

Her fonksiyon, transaction nesnesini geri döndürür, böylece zincirleme işlemler yapabilirsiniz:

```rust
let tx = transactional::begin(&mut client).await?;
let (tx, _) = transactional::tx_insert(tx, user1).await?;
let (tx, _) = transactional::tx_insert(tx, user2).await?;
let (tx, _) = transactional::tx_update(tx, activate_user).await?;
tx.commit().await?;
```
