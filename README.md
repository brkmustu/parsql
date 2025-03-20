# parsql

[![Version](https://img.shields.io/crates/v/parsql.svg)](https://crates.io/crates/parsql)
[![Documentation](https://docs.rs/parsql/badge.svg)](https://docs.rs/parsql)
[![License](https://img.shields.io/crates/l/parsql.svg)](https://github.com/yazdostum-nettr/parsql/blob/master/LICENSE)

Deneyimsel bir sql yardımcı küfesidir. Bu bir ORM aracı değildir. Amaç sql yazımı ve kullanımında basit cümlecikler için kolaylık sağlamaktır.

## Özellikler

- Otomatik SQL sorgu oluşturma
- Güvenli parametre yönetimi
- Birden fazla veritabanı sistemi için destek (PostgreSQL, SQLite, Tokio PostgreSQL, Deadpool PostgreSQL)
- Tip güvenliği olan veritabanı işlemleri
- SQL Injection saldırılarına karşı otomatik koruma
- **Yeni (0.3.3):** Sayfalama (pagination) için tam destek: `limit` ve `offset` öznitelikleri ile verimli sayfalama yapabilirsiniz
- `Queryable` türetme özniteliği, tablo adı, where ifadesi, select ifadesi, group by, having, order by, limit ve offset ifadeleri için destek sağlar.
- `Insertable` türetme özniteliği, tabloya özgü INSERT ifadeleri oluşturur.
- `Updateable` türetme özniteliği, tabloya özgü UPDATE ifadeleri oluşturur.
- `Deletable` türetme özniteliği, tabloya özgü DELETE ifadeleri oluşturur.
- `SqlParams` türetme özniteliği, yapının SQL parametreleri için kullanılmasını sağlar.
- `UpdateParams` türetme özniteliği, yapının UPDATE ifadeleri için kullanılmasını sağlar.
- `FromRow` türetme özniteliği, veritabanı satırlarının yapıya dönüştürülmesini sağlar.
- **Yeni (0.3.3):** SQL trace kayıtları için `PARSQL_TRACE` çevre değişkeni desteği eklendi.

## Ne İşe Yarar?

Parsql, SQL sorgularınızı doğrudan Rust struct'ları üzerinden yönetmenize olanak tanıyan bir kütüphanedir. Temel amacı, veritabanı işlemlerini daha güvenli ve daha az kod ile gerçekleştirmenizi sağlamaktır. Bu kütüphane ile:

- Struct tanımları üzerinden otomatik SQL sorguları oluşturabilirsiniz
- Veritabanı parametrelerini güvenli bir şekilde yönetebilirsiniz
- Generic CRUD işlemlerini (ekleme, okuma, güncelleme, silme) kolayca yapabilirsiniz
- Dinamik SQL oluşturabilir ve karmaşık sorgular çalıştırabilirsiniz
- Asenkron veritabanı işlemlerini kolayca gerçekleştirebilirsiniz
- SQL injection saldırılarına karşı otomatik koruma sağlayabilirsiniz
- Doğrudan `Pool` ve `Transaction` nesneleri üzerinde extension method'lar kullanabilirsiniz

Parsql standart bir ORM değildir. Daha çok, SQL yazımını ve kullanımını basitleştirmeye odaklanır.

## Desteklenen Veritabanları

Parsql aşağıdaki veritabanı sistemlerini desteklemektedir:

- **SQLite** (senkron): `parsql-sqlite` paketi
- **PostgreSQL** (senkron): `parsql-postgres` paketi
- **Tokio PostgreSQL** (asenkron): `parsql-tokio-postgres` paketi
- **Deadpool PostgreSQL** (asenkron bağlantı havuzu): `parsql-deadpool-postgres` paketi

## Kurulum

Cargo.toml dosyanıza şu şekilde ekleyin:

```toml
[dependencies]
parsql = "0.3.6"
```

ve özellik seçimini yapın:

```toml
# SQLite için
parsql = { version = "0.3.6", features = ["sqlite"] }

# PostgreSQL için
parsql = { version = "0.3.6", features = ["postgres"] }

# Tokio PostgreSQL için 
parsql = { version = "0.3.6", features = ["tokio-postgres"] }

# Deadpool PostgreSQL için
parsql = { version = "0.3.6", features = ["deadpool-postgres"] }
```

## Temel Özellikler

### Procedural Makrolar
Parsql, veritabanı işlemlerini kolaylaştırmak için çeşitli procedural makrolar sunar:

- `#[derive(Queryable)]` - Okuma (select) işlemleri için
- `#[derive(Insertable)]` - Ekleme işlemleri için
- `#[derive(Updateable)]` - Güncelleme işlemleri için
- `#[derive(Deletable)]` - Silme işlemleri için
- `#[derive(FromRow)]` - Veritabanı sonuçlarını nesnelere dönüştürmek için
- `#[derive(SqlParams)]` - SQL parametrelerini yapılandırmak için
- `#[derive(UpdateParams)]` - Güncelleme parametrelerini yapılandırmak için

### Extension Metodu Kullanımı

Parsql, 0.3.3 sürümünden itibaren, CRUD işlemlerini doğrudan veritabanı nesneleri üzerinden yapmanızı sağlayan extension metotları sunmaktadır. Bu yaklaşım sayesinde kodunuz daha akıcı ve okunabilir hale gelir.

#### Pool Nesnesi Üzerinde Extension Metodları

Bağlantı havuzu (Pool) nesneleri üzerinde doğrudan CRUD işlemleri yapabilirsiniz:

```rust
// Geleneksel kullanım
let rows_affected = insert(&pool, user).await?;

// Extension metodu ile kullanım
use parsql_deadpool_postgres::CrudOps;
let rows_affected = pool.insert(user).await?;
```

#### Transaction Nesnesi Üzerinde Extension Metodları

Transaction nesneleri üzerinde doğrudan CRUD işlemleri yapabilirsiniz:

```rust
// Geleneksel kullanım
let (tx, rows_affected) = tx_insert(tx, user).await?;

// Extension metodu ile kullanım
use parsql_deadpool_postgres::TransactionOps;
let rows_affected = tx.insert(user).await?;
```

#### Desteklenen Extension Metodları

Hem Pool hem de Transaction nesneleri için şu extension metodları kullanılabilir:

- `insert(entity)` - Kayıt ekler
- `update(entity)` - Kayıt günceller
- `delete(entity)` - Kayıt siler
- `fetch(params)` - Tek bir kayıt getirir
- `fetch_all(params)` - Birden fazla kayıt getirir
- `select(entity, to_model)` - Özel dönüştürücü fonksiyon ile tek kayıt getirir
- `select_all(entity, to_model)` - Özel dönüştürücü fonksiyon ile çoklu kayıt getirir

### Transaction Desteği

Parsql şu anda aşağıdaki paketlerde transaction desteği sunmaktadır:

- `parsql-postgres` - Senkron PostgreSQL işlemleri için transaction desteği
- `parsql-tokio-postgres` - Asenkron Tokio-PostgreSQL işlemleri için transaction desteği
- `parsql-deadpool-postgres` - Asenkron Deadpool PostgreSQL bağlantı havuzu için transaction desteği

Örnek bir transaction kullanımı:

```rust
// Transaction başlatma
let client = pool.get().await?;
let tx = client.transaction().await?;

// Extension method kullanarak transaction içinde işlem yapma
let result = tx.insert(user).await?;
let rows_affected = tx.update(user_update).await?;

// İşlem başarılı olursa commit
tx.commit().await?;
```

### Güvenlik Özellikleri

#### SQL Injection Koruması
Parsql, SQL injection saldırılarına karşı güvenli bir şekilde tasarlanmıştır:

- Parametreli sorgular otomatik olarak kullanılır, asla direk string birleştirme yapılmaz
- Tüm kullanıcı girdileri güvenli bir şekilde parametrize edilir
- Makrolar, SQL parametrelerini doğru bir şekilde işler ve güvenli bir format sağlar
- Her veritabanı adaptörü için uygun parametre işaretleyiciler (`$1`, `?`, vb.) otomatik olarak uygulanır
- SQL yazarken elle string birleştirme gereksinimi ortadan kaldırılmıştır
- Asenkron bağlamlarda bile güvenlik önlemleri tam olarak korunur

```rust
// Güvenli parametre kullanımı örneği
#[derive(Queryable, FromRow, SqlParams)]
#[table("users")]
#[where_clause("username = $ AND status = $")]
struct UserQuery {
    username: String,
    status: i32,
}

// Parametreler güvenli bir şekilde yerleştirilir, 
// SQL injection riski olmaz
let query = UserQuery {
    username: user_input,
    status: 1,
};
```

### Öznitelikler
Sorgularınızı özelleştirmek için çeşitli öznitelikler kullanabilirsiniz:

- `#[table("tablo_adi")]` - Tablo adını belirtmek için
- `#[where_clause("id = $")]` - WHERE koşulunu belirtmek için
- `#[select("alan1, alan2")]` - SELECT ifadesini özelleştirmek için
- `#[update("alan1, alan2")]` - UPDATE ifadesini özelleştirmek için
- `#[join("LEFT JOIN tablo2 ON tablo1.id = tablo2.fk_id")]` - JOIN ifadeleri için
- `#[group_by("alan1")]` - GROUP BY ifadesi için
- `#[order_by("alan1 DESC")]` - ORDER BY ifadesi için
- `#[having("COUNT(*) > 5")]` - HAVING ifadesi için
- `#[limit(10)]` - LIMIT ifadesi için
- `#[offset(5)]` - OFFSET ifadesi için
- `#[returning("id")]` - INSERT/UPDATE işlemlerinden dönen değerleri belirtmek için

### SQL İzleme
Geliştirme sırasında oluşturulan SQL sorgularını izlemek için:

```sh
PARSQL_TRACE=1 cargo run
```

Bu, çalıştırılan tüm SQL sorgularını konsola yazdıracaktır.

## Basit Kullanım Örnekleri

### SQLite ile Kullanım

```rust
use parsql::{
    sqlite::{fetch, insert},
    macros::{Queryable, Insertable, FromRow, SqlParams},
};
use rusqlite::Connection;

// Bir kayıt almak için
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

// Yeni kayıt eklemek için
#[derive(Insertable, SqlParams)]
#[table("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("test.db")?;
    
    let insert_user = InsertUser {
        name: "Ali".to_string(),
        email: "ali@example.com".to_string(),
    };
    
    let id = insert(&conn, insert_user)?;
    println!("Eklenen kayıt ID: {}", id);
    
    let get_user = GetUser::new(id);
    let user = fetch(&conn, get_user)?;
    println!("Kullanıcı: {:?}", user);
    
    Ok(())
}
```

### Deadpool PostgreSQL ile Asenkron Bağlantı Havuzu Kullanımı

```rust
use parsql_deadpool_postgres::{CrudOps, TransactionOps};
use tokio_postgres::NoTls;
use deadpool_postgres::{Config, Runtime};
use parsql_macros::{Queryable, Insertable, FromRow, SqlParams, Updateable};

#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("id = $")]
pub struct GetUser {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[derive(Insertable, SqlParams)]
#[table("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
}

#[derive(Updateable, SqlParams)]
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
    // Bağlantı havuzu oluşturma
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.user = Some("postgres".to_string());
    cfg.password = Some("postgres".to_string());
    cfg.dbname = Some("test".to_string());
    
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
    
    // Extension method kullanarak kayıt ekleme
    let insert_user = InsertUser {
        name: "Ali".to_string(),
        email: "ali@example.com".to_string(),
    };
    let rows_affected = pool.insert(insert_user).await?;
    println!("Eklenen kayıt sayısı: {}", rows_affected);
    
    // Transaction kullanımı
    let client = pool.get().await?;
    let tx = client.transaction().await?;
    
    // Transaction içinde extension method kullanarak güncelleme
    let update_user = UpdateUser {
        id: 1,
        name: "Ali Güncellendi".to_string(),
        email: "ali.updated@example.com".to_string(),
    };
    let rows_affected = tx.update(update_user).await?;
    
    // Başarılı olursa commit
    tx.commit().await?;
    
    Ok(())
}
```

## Performans İpuçları

- Aynı SQL yapısına sahip sorguları tekrar kullanarak sorgu planı ön belleğinden yararlanın
- Yoğun veritabanı uygulamaları için bağlantı havuzları kullanın
- Büyük veri kümeleri için `get_all` yerine sayfalama (limit ve offset) kullanın
- Filtreleri veritabanı seviyesinde uygulayın, uygulamanızda değil

## Detaylı Dökümantasyon

Her veritabanı adaptörü için daha detaylı bilgi ve örnekler, ilgili alt paketlerin README dosyalarında bulunmaktadır:

- [SQLite Dökümantasyonu](./parsql-sqlite/README.md)
- [PostgreSQL Dökümantasyonu](./parsql-postgres/README.md)
- [Tokio PostgreSQL Dökümantasyonu](./parsql-tokio-postgres/README.md)
- [Deadpool PostgreSQL Dökümantasyonu](./parsql-deadpool-postgres/README.md)

## Lisans

Bu proje MIT lisansı altında lisanslanmıştır.
