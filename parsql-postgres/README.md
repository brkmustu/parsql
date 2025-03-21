# parsql-postgres

Parsql için PostgreSQL entegrasyon küfesidir. Bu paket, parsql'in PostgreSQL veritabanlarıyla çalışmasını sağlayan **senkron** API'leri içerir.

## Özellikler

- Senkron PostgreSQL işlemleri
- Otomatik SQL sorgu oluşturma 
- Güvenli parametre yönetimi
- Generic CRUD işlemleri (get, insert, update, delete)
- Client nesnesi için extension metotları
- Transaction desteği
- Veritabanı satırlarını struct'lara dönüştürme
- SQL Injection saldırılarına karşı otomatik koruma

## Güvenlik Özellikleri

### SQL Injection Koruması

parsql-postgres, SQL Injection saldırılarına karşı güvenli bir şekilde tasarlanmıştır:

- Tüm kullanıcı girdileri otomatik olarak parametrize edilir
- PostgreSQL'in "$1, $2, ..." parametrelendirme yapısı otomatik olarak kullanılır
- Makrolar, SQL parametrelerini güvenli bir şekilde işleyerek injection saldırılarına karşı koruma sağlar
- Parametrelerin doğru sırada ve tipte gönderilmesi otomatik olarak yönetilir
- `#[where_clause]` ve diğer SQL bileşenlerinde kullanıcı girdileri her zaman parametrize edilir

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
let user = get(&conn, query)?;
```

## Kurulum

Cargo.toml dosyanıza şu şekilde ekleyin:

```toml
[dependencies]
parsql = { version = "0.3.7", features = ["postgres"] }
```

veya doğrudan bu paketi kullanmak isterseniz:

```toml
[dependencies]
parsql-postgres = "0.3.2"
parsql-macros = "0.3.2"
postgres = "0.19"
```

## Kullanım

parsql-postgres ile çalışmak için iki farklı yaklaşım kullanabilirsiniz:

### 1. Fonksiyon Tabanlı Yaklaşım

```rust
use postgres::{Client, NoTls};
use parsql::postgres::{get, insert};
use parsql::macros::{Insertable, SqlParams, Queryable, FromRow};

#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
}

#[derive(Queryable, FromRow, SqlParams)]
#[table("users")]
#[where_clause("id = $1")]
struct GetUser {
    id: i32,
    name: String,
    email: String,
}

fn main() -> Result<(), postgres::Error> {
    let mut client = Client::connect("host=localhost user=postgres", NoTls)?;
    
    // Fonksiyon yaklaşımı ile kullanıcı ekleme
    let insert_user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    let rows_affected = insert(&mut client, insert_user)?;
    
    // Fonksiyon yaklaşımı ile kullanıcı getirme
    let get_user = GetUser {
        id: 1,
        name: String::new(),
        email: String::new(),
    };
    let user = get(&mut client, &get_user)?;
    
    println!("Kullanıcı: {:?}", user);
    Ok(())
}
```

### 2. Extension Metot Yaklaşımı (CrudOps Trait)

Bu yaklaşımda, `CrudOps` trait'i sayesinde CRUD işlemlerini doğrudan `Client` nesnesi üzerinden çağırabilirsiniz:

```rust
use postgres::{Client, NoTls};
use parsql::postgres::CrudOps;  // CrudOps trait'ini içe aktar
use parsql::macros::{Insertable, SqlParams, Queryable, FromRow};

#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
}

#[derive(Queryable, FromRow, SqlParams)]
#[table("users")]
#[where_clause("id = $1")]
struct GetUser {
    id: i32,
    name: String,
    email: String,
}

fn main() -> Result<(), postgres::Error> {
    let mut client = Client::connect("host=localhost user=postgres", NoTls)?;
    
    // Extension metot yaklaşımı ile kullanıcı ekleme
    let insert_user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    let rows_affected = client.insert(insert_user)?;
    
    // Extension metot yaklaşımı ile kullanıcı getirme
    let get_user = GetUser {
        id: 1,
        name: String::new(),
        email: String::new(),
    };
    let user = client.get(&get_user)?;
    
    println!("Kullanıcı: {:?}", user);
    Ok(())
}
```

Extension metot yaklaşımı, özellikle birden fazla CRUD işleminin aynı anda yapıldığı durumlarda kodunuzu daha okunabilir ve akıcı hale getirir.

## Temel Kullanım

Bu paket, PostgreSQL veritabanı ile çalışırken **senkron işlemler** kullanır. Bu, tokio-postgres gibi async/await kullanımı gerektirmediği anlamına gelir.

### Bağlantı Kurma

```rust
use postgres::{Client, NoTls, Error};

fn main() -> Result<(), Error> {
    // PostgreSQL bağlantısı oluşturma
    let mut client = Client::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    )?;
    
    // Örnek tablo oluşturma
    client.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            state SMALLINT NOT NULL
        )",
        &[],
    )?;
    
    // ...
    
    Ok(())
}
```

## CRUD İşlemleri

### Veri Okuma (Get) İşlemi

```rust
use parsql::{
    core::Queryable,
    macros::{FromRow, Queryable, SqlParams},
    postgres::{FromRow, SqlParams, get},
};
use postgres::{types::ToSql, Row};

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    )?;
    
    // Kullanımı
    let get_user = GetUser::new(1);
    let get_result = get(&mut client, get_user)?;
    
    println!("Kullanıcı: {:?}", get_result);
    Ok(())
}
```

### Veri Ekleme (Insert) İşlemi

```rust
use parsql::{
    core::Insertable,
    macros::{Insertable, SqlParams},
    postgres::{SqlParams, insert},
};
use postgres::types::ToSql;

#[derive(Insertable)]
#[table("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect(
        "host=localhost user=postgres password=postgres dbname=test",
        NoTls,
    )?;
    
    let insert_user = InsertUser {
        name: "Ali".to_string(),
        email: "ali@parsql.com".to_string(),
        state: 1,
    };
    
    let insert_result = insert(&mut client, insert_user)?;
    println!("Eklenen kayıt ID: {}", insert_result);
    
    Ok(())
}
```

### Veri Güncelleme (Update) İşlemi

```

## Transaction İşlemleri

parsql-postgres ile transaction işlemlerini iki farklı şekilde gerçekleştirebilirsiniz:

### 1. CrudOps Trait'i ile Transaction Kullanımı

Bu yaklaşımda, `CrudOps` trait'i `Transaction` struct'ı için de implemente edilmiştir, böylece doğrudan transaction nesnesi üzerinden CRUD işlemlerini yapabilirsiniz:

```rust
use postgres::{Client, NoTls};
use parsql::postgres::CrudOps;  // CrudOps trait'ini içe aktar
use parsql::macros::{Insertable, SqlParams, Updateable, UpdateParams};

#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
}

#[derive(Updateable, UpdateParams)]
#[table("users")]
#[update("email")]
#[where_clause("id = $")]
struct UpdateUser {
    id: i32,
    email: String,
}

fn main() -> Result<(), postgres::Error> {
    let mut client = Client::connect("host=localhost user=postgres", NoTls)?;
    
    // Transaction başlat
    let mut tx = client.transaction()?;
    
    // Transaction üzerinde CrudOps metotlarını kullan
    let insert_user = InsertUser {
        name: "Ali".to_string(),
        email: "ali@example.com".to_string(),
    };
    let rows_affected = tx.insert(insert_user)?;
    
    let update_user = UpdateUser {
        id: 1,
        email: "ali.updated@example.com".to_string(),
    };
    let rows_updated = tx.update(update_user)?;
    
    // Transaction'ı tamamla
    tx.commit()?;
    Ok(())
}
```

### 2. Transaction Yardımcı Fonksiyonları ile Kullanım

Bu yaklaşımda, `transactional` modülündeki yardımcı fonksiyonları kullanarak method chaining yaklaşımıyla işlemlerinizi gerçekleştirebilirsiniz:

```rust
use postgres::{Client, NoTls};
use parsql::postgres::transactional::{begin, tx_insert, tx_update};
use parsql::macros::{Insertable, SqlParams, Updateable, UpdateParams};

#[derive(Insertable, SqlParams)]
#[table("users")]
struct InsertUser {
    name: String,
    email: String,
}

#[derive(Updateable, UpdateParams)]
#[table("users")]
#[update("email")]
#[where_clause("id = $")]
struct UpdateUser {
    id: i32,
    email: String,
}

fn main() -> Result<(), postgres::Error> {
    let mut client = Client::connect("host=localhost user=postgres", NoTls)?;
    
    // Transaction başlat
    let tx = begin(&mut client)?;
    
    // Transaction işlemlerini zincirleme (method chaining) şeklinde gerçekleştir
    let insert_user = InsertUser {
        name: "Ali".to_string(),
        email: "ali@example.com".to_string(),
    };
    
    let (tx, _) = tx_insert(tx, insert_user)?;
    
    let update_user = UpdateUser {
        id: 1,
        email: "ali.updated@example.com".to_string(),
    };
    
    let (tx, _) = tx_update(tx, update_user)?;
    
    // Transaction'ı tamamla
    tx.commit()?;
    Ok(())
}
```

Bu yaklaşım, özellikle transaction içinde birden fazla işlem gerçekleştirirken, transaction nesnesinin sürekli olarak elde edilebilmesini sağlar ve kod okunabilirliğini artırır.