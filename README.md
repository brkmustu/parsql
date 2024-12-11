# parsql
Deneyimsel sql yardımcısı

### Ne işe yarar?

Temel sql cümleciklerinin direkt "struct" üzerinden yönetilebilmesini sağlayacak, "generic crud" işlemlerini kullanılabilir hale getiren yardımcı makro, trait ve fonksiyonlar içerir.

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

