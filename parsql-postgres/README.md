# parsql-postgres

parsql için postgres entegrasyon küfesidir.

### Genel Kullanım Örnekleri

```rust
#[derive(Insertable)]
#[table_name("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}
```

yukarıdaki gibi bir struct tanımlaması yaptıktan sonra eklemeniz gereken toplam 5 adet bağımlılık söz konusu;

```rust
use parsql::{core::Insertable, macros::Insertable, postgres::{insert, SqlParams}};
use postgres::types::ToSql;
```

bunlar mevcut küfede bulunan makro, trait ve generic fonksiyonlardan faydalanmanızı sağlayacak.

Sonrasında aşağıdaki gibi bir kullanım ile insert işleminizi gerçekleştirebilirsiniz;

```rust
    let insert_usert = InsertUser {
        name: "Ali".to_string(),
        email: "ali@parsql.com".to_string(),
        state: 1,
    };

    let insert_result = insert(&conn, insert_usert);
```

hepsi bu kadar.
