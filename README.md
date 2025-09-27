# parsql

[![Version](https://img.shields.io/crates/v/parsql.svg)](https://crates.io/crates/parsql)
[![Documentation](https://docs.rs/parsql/badge.svg)](https://docs.rs/parsql)
[![License](https://img.shields.io/crates/l/parsql.svg)](https://github.com/yazdostum-nettr/parsql/blob/master/LICENSE)

Deneyimsel bir sql yardımcı küfesidir. Bu bir ORM aracı değildir. Amaç sql yazımı ve kullanımında basit cümlecikler için kolaylık sağlamaktır.

## Özellikler

### 🚀 Core Library
- **Otomatik SQL sorgu oluşturma** - Struct'lardan SQL generate etme
- **Güvenli parametre yönetimi** - SQL Injection saldırılarına karşı otomatik koruma
- **Multi-database destek** - PostgreSQL, SQLite, Tokio PostgreSQL, Deadpool PostgreSQL
- **Tip güvenliği** - Compile-time type safety
- **Sayfalama desteği** - `limit` ve `offset` öznitelikleri ile verimli pagination

### 🎯 Derive Macros
- `#[derive(Queryable)]` - SELECT işlemleri (where, select, group by, having, order by, limit, offset)
- `#[derive(Insertable)]` - INSERT işlemleri
- `#[derive(Updateable)]` - UPDATE işlemleri  
- `#[derive(Deletable)]` - DELETE işlemleri
- `#[derive(FromRow)]` - Row-to-struct conversion
- `#[derive(SqlParams, UpdateParams)]` - Parameter handling

### 🛠️ CLI ve Migration Sistemi (v0.5.0+)
- **🎨 Interactive TUI** - Modern ve kullanıcı dostu terminal arayüzü
- **📋 Command Line Interface** - Automation ve scripting için
- **🔄 Migration Management** - Create, run, rollback, status tracking
- **✅ Transaction Safety** - Her migration kendi transaction'ında
- **🔍 Checksum Verification** - Modified migration detection
- **🚫 Gap Detection** - Missing migration protection
- **📊 Real-time Status** - Live migration tracking
- **🎯 Smart Auto-completion** - TUI'de akıllı komut tamamlama

### 🔧 Advanced Features
- **SQL trace logging** - `PARSQL_TRACE` environment variable
- **Extension methods** - Pool ve Transaction nesneleri üzerinde direct usage
- **Prelude module** - Tek import ile tüm gerekli traits
- **Async support** - Tokio ve Deadpool integration

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

Cargo.toml içinde aşağıdaki şekilde tanımlama yapın:

```toml
[dependencies]
parsql = { version = "0.5.0", features = ["sqlite"] }
```

veya PostgreSQL için:

```toml
[dependencies]
parsql = { version = "0.5.0", features = ["postgres"] }
```

veya Tokio PostgreSQL için:

```toml
[dependencies]
parsql = { version = "0.5.0", features = ["tokio-postgres"] }
```

veya Deadpool PostgreSQL için:

```toml
[dependencies]
parsql = { version = "0.5.0", features = ["deadpool-postgres"] }
```

## Temel Özellikler

### Prelude Modülü (v0.5.0+)

Parsql artık tüm yaygın kullanılan trait'ler, makrolar ve tipleri içeren bir `prelude` modülü sunmaktadır:

```rust
use parsql::prelude::*;
```

Bu import ile şunlara erişebilirsiniz:
- Tüm derive makroları (`Queryable`, `Insertable`, `Updateable`, `Deletable`, `FromRow`, `SqlParams`, `UpdateParams`)
- Tüm trait'ler (`CrudOps`, `FromRow`, `SqlParams`, `SqlQuery`, `SqlCommand`, `UpdateParams`)
- Veritabanına özel tipler (`Row`, `ToSql`, `Connection`, `Error`, vb.)
- Extension trait'leri (`SqliteConnectionExt`, `PostgresConnectionExt`, vb.)

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

// Extension metodu ile kullanım (prelude ile otomatik gelir)
let rows_affected = pool.insert(user).await?;
```

#### Transaction Nesnesi Üzerinde Extension Metodları

Transaction nesneleri üzerinde doğrudan CRUD işlemleri yapabilirsiniz:

```rust
// Geleneksel kullanım
let (tx, rows_affected) = tx_insert(tx, user).await?;

// Extension metodu ile kullanım (prelude ile otomatik gelir)
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
use parsql::prelude::*;

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
use parsql::prelude::*;
use deadpool_postgres::{Config, Runtime};

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

## Veritabanı Migrasyonları

Parsql, basit ve tip güvenli bir migrasyon sistemi içerir:

```rust
use parsql::prelude::*;

// Migration tanımlama
pub struct CreateUsersTable;

impl Migration for CreateUsersTable {
    fn version(&self) -> i64 { 20240101120000 }
    fn name(&self) -> &str { "create_users_table" }
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<()> {
        conn.execute("CREATE TABLE users (id SERIAL PRIMARY KEY, email VARCHAR(255))")
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<()> {
        conn.execute("DROP TABLE users")
    }
}

// Migration çalıştırma
let mut runner = MigrationRunner::new();
runner.add_migration(Box::new(CreateUsersTable));
runner.run(&mut conn)?;
```

## CLI Aracı ve Migration Sistemi

Parsql CLI, migration yönetimi için gelişmiş komut satırı aracı ve interaktif TUI (Terminal User Interface) sunar.

### Kurulum

```bash
cargo install parsql-cli
```

### İki Kullanım Modu

#### 1. Interactive TUI Mode (Önerilen)

```bash
# Interaktif TUI başlatma
parsql
# veya
parsql -i
```

**TUI Özellikleri:**
- 🎨 **Modern ve sezgisel terminal arayüzü**
- 📊 **Gerçek zamanlı migration durumu görüntüleme**
- 🔄 **Canlı log takibi ve progress göstergeleri**
- ⌨️ **Akıllı komut tamamlama**
- 🎯 **Kolay navigasyon ve hızlı aksiyonlar**

**Navigasyon:**
- `Tab`: Görünümler arası geçiş (Migrations, Logs, Config)
- `↑/↓` veya `j/k`: Liste navigasyonu
- `Enter`: Seçim/açma
- `ESC` veya `q`: Geri gitme
- `/`: Komut modu (command palette)

**TUI Komutları (`/` tuşu ile):**
- `/help` - Yardım göster
- `/connect <url>` - Veritabanına bağlan
- `/create <name>` - Yeni migration oluştur
- `/run` - Bekleyen migration'ları çalıştır
- `/rollback <version>` - Belirtilen versiyona geri al
- `/status` - Migration durumunu göster
- `/validate` - Migration'ları doğrula
- `/list` - Migration listesi
- `/config` - Konfigürasyon görüntüle
- `/refresh` - Veriyi yenile
- `/quit` - Çıkış

**TUI Görünümleri:**

1. **Migration List View (Ana Görünüm)**
   - Tüm migration'ları durum ile listeler
   - ✅ Applied (Uygulandı) - Yeşil
   - ⏳ Pending (Bekliyor) - Sarı  
   - ❌ Failed (Başarısız) - Kırmızı
   - Hızlı aksiyonlar: `r` (yenile), `a` (tümünü uygula)

2. **Migration Detail View**
   - Seçili migration'ın SQL içeriğini gösterir
   - Syntax highlighting ile kolay okuma
   - Aksiyonlar: `r` (çalıştır), `b` (geri al)
   - Satır numaraları ve dosya bilgisi

3. **Migration Content Viewer**
   - Up/down migration dosyalarını görüntüleme
   - SQL syntax highlighting
   - Dosya editörü entegrasyonu

4. **Logs View**
   - Gerçek zamanlı uygulama logları
   - Seviye bazlı renk kodlama:
     - 🔵 INFO - Mavi
     - 🟡 WARN - Sarı
     - 🔴 ERROR - Kırmızı
     - 🟢 SUCCESS - Yeşil

5. **Configuration View**
   - Mevcut veritabanı bağlantısı
   - Migration ayarları
   - Dosya yolları ve konfigürasyon detayları

6. **Database Connection View**
   - Bağlantı durumu görüntüleme
   - Veritabanı bilgileri (tip, versiyon, tablo sayısı)
   - Bağlantı testi ve durum kontrolü

#### 2. Command Line Mode

```bash
# Proje başlatma
parsql init

# Migration oluşturma
parsql migrate create "create users table" --migration-type sql

# Migration çalıştırma
parsql migrate run --database-url postgresql://localhost/mydb

# Durum kontrolü
parsql migrate status --detailed

# Geri alma
parsql migrate rollback --to 20240101000000

# Doğrulama
parsql migrate validate --verify-checksums

# Migration listesi
parsql migrate list --pending
```

### Konfigürasyon

`parsql.toml` dosyası oluşturun:

```toml
[database]
url = "postgresql://user:pass@localhost/dbname"
# veya SQLite için:
# url = "sqlite:app.db"

[migrations]
directory = "migrations"
table_name = "schema_migrations"
verify_checksums = true
allow_out_of_order = false
transaction_per_migration = true
```

### Çevre Değişkenleri

```bash
export DATABASE_URL="postgresql://localhost/mydb"
export PARSQL_MIGRATIONS_DIR="custom_migrations"
export PARSQL_CONFIG="config/parsql.toml"
```

### Praktik Kullanım Senaryoları

#### Senaryo 1: Blog Projesi (PostgreSQL)

```bash
# 1. Yeni blog projesi başlat
parsql init --database-url postgresql://localhost/blog

# 2. İlk migration: Users tablosu
parsql migrate create "create_users_table"
# migrations/20240101120000_create_users_table.up.sql oluşturuldu

# 3. SQL dosyasını düzenle:
cat > migrations/20240101120000_create_users_table.up.sql << EOF
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
EOF

# 4. Down migration:
cat > migrations/20240101120000_create_users_table.down.sql << EOF
DROP TABLE IF EXISTS users;
EOF

# 5. Migration'ı çalıştır
parsql migrate run

# 6. Posts tablosu ekle
parsql migrate create "create_posts_table"
cat > migrations/20240101130000_create_posts_table.up.sql << EOF
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    published BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_posts_user_id ON posts(user_id);
CREATE INDEX idx_posts_published ON posts(published);
EOF

# 7. Tüm migration'ları çalıştır
parsql migrate run

# 8. Durum kontrolü
parsql migrate status --detailed
```

#### Senaryo 2: E-ticaret Projesi (SQLite)

```bash
# 1. E-ticaret projesi
parsql init --database-url sqlite:ecommerce.db

# 2. İnteraktif modda çalış
parsql -i

# TUI'de yapılacaklar:
# - Tab ile Migration List view'a git
# - / tuşuna basıp komut modunu aç
# - /create products_table yazıp Enter
# - Migration dosyası düzenledikten sonra:
# - /run komutu ile migration'ı çalıştır
# - Tab ile Logs view'a geçip sonuçları gör
```

#### Senaryo 3: Microservice Migration (Docker)

```bash
# 1. Docker container içinde migration
docker run --rm -v $(pwd):/app -w /app \
  --network host \
  parsql-cli:latest migrate run \
  --database-url postgresql://postgres:password@localhost:5432/microservice

# 2. CI/CD pipeline integration
parsql migrate validate --verify-checksums
if [ $? -eq 0 ]; then
  parsql migrate run --database-url $DATABASE_URL
  parsql migrate status --detailed
fi

# 3. Production deployment check
parsql migrate status --database-url $PROD_DATABASE_URL
parsql migrate run --dry-run --database-url $PROD_DATABASE_URL
```

#### Senaryo 4: Development Workflow

```bash
# 1. Geliştirme ortamı hazırlığı
export DATABASE_URL="postgresql://dev:dev@localhost:5432/myapp_dev"
parsql migrate run

# 2. Yeni feature için migration
git checkout -b feature/user-profiles
parsql migrate create "add_user_profiles"

# 3. Migration geliştirme ve test
parsql -i  # TUI'de SQL dosyasını düzenle ve test et

# 4. Test veritabanında deneme
export DATABASE_URL="postgresql://test:test@localhost:5432/myapp_test"
parsql migrate run

# 5. Production'a hazırlık
parsql migrate validate --verify-checksums --check-gaps
parsql migrate run --dry-run --database-url $STAGING_DATABASE_URL

# 6. Rollback planı
parsql migrate status --detailed
# Eğer bir problem olursa:
# parsql migrate rollback --to <previous_version>
```

#### Senaryo 5: TUI'de İnteraktif Kullanım

```bash
# TUI başlat
parsql -i

# TUI Workflow:
# 1. Tab ile farklı görünümler arasında gezin
# 2. / tuşu ile komut modunu açın
# 3. /connect postgresql://localhost/mydb - veritabanına bağlanın
# 4. /create "table_name" - yeni migration oluşturun
# 5. Enter ile migration'ı seçin ve SQL içeriğini görün
# 6. /run - migration'ı çalıştırın
# 7. Tab ile Logs view'a geçin ve sonuçları takip edin
# 8. /status - genel durumu kontrol edin
# 9. Ctrl+Q ile çıkın
```

### Troubleshooting

```bash
# Migration başarısız oldu mu?
parsql migrate status --detailed
parsql migrate rollback --to <last_good_version>

# Checksum hatası var mı?
parsql migrate validate --verify-checksums

# Connection sorunları
parsql -i  # TUI'de /connect komutu ile test edin

# Log takibi
parsql -i  # TUI'de Logs view'ını kullanın
```

## Detaylı Dökümantasyon

Her veritabanı adaptörü için daha detaylı bilgi ve örnekler, ilgili alt paketlerin README dosyalarında bulunmaktadır:

- [SQLite Dökümantasyonu](./parsql-sqlite/README.md)
- [PostgreSQL Dökümantasyonu](./parsql-postgres/README.md)
- [Tokio PostgreSQL Dökümantasyonu](./parsql-tokio-postgres/README.md)
- [Deadpool PostgreSQL Dökümantasyonu](./parsql-deadpool-postgres/README.md)
- [Migration Sistemi](./parsql-migrations/README.md)
- [CLI Aracı](./parsql-cli/README.md)

## Lisans

Bu proje MIT lisansı altında lisanslanmıştır.
