# parsql
Deneyimsel sql yardımcısı

### Küfe'nin yüklenmesi

Uygulamanıza küfeyi yüklerken hangi veritabanı ile çalışacağınızı 'feature' olarak belirtmeniz gerekiyor. Örneğin 'postgresql' ile çalışacaksanız ve 'tokio' runtime kullanıyorsanız, Cargo.toml dosyanıza paketi aşağıdaki şekilde eklemeniz gerekiyor;

```rust
parsql = { version = "0.1.1", features = ["tokio-postgres"] }
```

### Ne işe yarar?

Temel sql cümleciklerinin direkt "struct" üzerinden yönetilebilmesini sağlayacak, küfe içindeki "generic crud" işlemlerini kullanılabilir hale getiren yardımcı makro, trait ve fonksiyonlar içerir.

Örneğin;

```rust
#[derive(Queryable, Debug)]
#[table_name("users")]
#[where_clause("id = $")]
pub struct GetUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}
```

gibi bir procedural makro kullanımı ile, desteklenen (şimdilik sqlite ve postgresql) veritabanlarında küfe'de tanımlanan "get" fonksiyonunu, bu "struct" için uygulayabilir hale getirmiş oluyoruz.

Şunun gibi;

```rust
    let get_user_result = get(&conn, get_user, |row| {
        Ok(GetUser {
            id: row.get("id").unwrap(),
            name: row.get("name").unwrap(),
            email: row.get("email").unwrap(),
            state: row.get("state").unwrap(),
        })
    });

    println!("get user result: {:?}", get_user_result);
```

github'da projenin repository'sinde, "examples" klasörü altında "sqlite" ve "tokio-postgres" örnek projelerinde, örnek kullanımlar mevcuttur.
