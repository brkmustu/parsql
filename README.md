# parsql
Deneyimsel sql yardımcısı

### Küfe'nin yüklenmesi

Uygulamanıza küfeyi yüklerken hangi veritabanı ile çalışacağınızı 'feature' olarak belirtmeniz gerekiyor. Örneğin 'postgresql' ile çalışacaksanız ve 'tokio' runtime kullanıyorsanız, Cargo.toml dosyanıza paketi aşağıdaki şekilde eklemeniz gerekiyor;

```rust
parsql = { version = "0.2.0", features = ["tokio-postgres"] }
```

### Ne işe yarar?

Temel sql cümleciklerinin direkt "struct" üzerinden yönetilebilmesini sağlayacak, küfe içindeki "generic crud" işlemlerini kullanılabilir hale getiren yardımcı makro, trait ve fonksiyonlar içerir.

Örneğin;

```rust
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
```

gibi bir procedural makro kullanımı ile, desteklenen (şimdilik sqlite ve postgresql) veritabanlarında küfe'de tanımlanan "get" fonksiyonunu, bu "struct" için uygulayabilir hale getirmiş oluyoruz.

yukarıdaki gibi bir struct tanımlaması yaptıktan sonra eklemeniz gereken toplam 8 adet bağımlılık söz konusu (aslında sadece get işleminde bu kadar çok bağımlılık var, diğerlerinde 5 adet bağımlılık ile generic fonksiyon kullanılabiliyor);

```rust
use parsql::{
    core::Queryable,
    macros::{FromRow, Queryable, SqlParams},
    tokio_postgres::{FromRow, SqlParams},
};
use tokio_postgres::{types::ToSql, Row};
```

Şunun gibi;

```rust
    let get_user = GetUser::new(1);
    let get_result = get(&client, get_user).await;

    println!("get user result: {:?}", get_user_result);
```

github'da projenin repository'sinde, "examples" klasörü altında "sqlite" ve "tokio-postgres" örnek projelerinde, örnek kullanımlar mevcuttur.

### Gruplama ve Sıralama Özellikleri

Sorgularınızda GROUP BY ve ORDER BY ifadelerini kullanmak için `group_by` ve `order_by` özniteliklerini kullanabilirsiniz:

```rust
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[select("users.state, COUNT(*) as user_count")]
#[group_by("users.state")]
#[order_by("user_count DESC")]
pub struct UserStateStats {
    pub state: i16,
    pub user_count: i64,
}
```

Bu öznitelikler opsiyoneldir ve ihtiyacınıza göre birlikte veya ayrı ayrı kullanılabilir. Örneğin:

```rust
// Sadece sıralama
#[derive(Queryable, FromRow, SqlParams)]
#[table("users")]
#[order_by("created_at DESC")]
pub struct RecentUsers { ... }

// Join ve gruplama birlikte
#[derive(Queryable, FromRow, SqlParams)]
#[table("users")]
#[select("users.state, COUNT(posts.id) as post_count")]
#[join("LEFT JOIN posts ON users.id = posts.user_id")]
#[group_by("users.state")]
pub struct UserPostCounts { ... }
```
