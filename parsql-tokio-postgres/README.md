# parsql-postgres

parsql için postgres entegrasyon küfesidir.

### Genel Kullanım Örnekleri

#### Generic 'insert' Kullanımı

Generic insert işlemini kullanabilmek için struct tanımınıza aşağıdaki gibi derive makrolarını eklemelisiniz.

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
use parsql::{
    core::Insertable,
    macros::{Insertable, SqlParams},
    tokio_postgres::SqlParams,
};
use tokio_postgres::types::ToSql;
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

#### Generic 'get' Kullanımı

Generic 'get' işlemini kullanabilmek için struct tanımınıza aşağıdaki gibi derive makrolarını eklemelisiniz.

```rust
#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table_name("users")]
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

yukarıdaki gibi bir struct tanımlaması yaptıktan sonra eklemeniz gereken toplam 8 adet bağımlılık söz konusu;

```rust
use parsql::{
    core::Queryable,
    macros::{FromRow, Queryable, SqlParams},
    tokio_postgres::{FromRow, SqlParams},
};
use tokio_postgres::{types::ToSql, Row};
```

bunlar mevcut küfede bulunan makro, trait ve generic fonksiyonlardan faydalanmanızı sağlayacak.

Sonrasında aşağıdaki gibi bir kullanım ile 'get' işleminizi gerçekleştirebilirsiniz;

```rust
    let get_user = GetUser::new(24025);
    let get_result = get(&client, get_user).await;
```

#### Generic 'update' Kullanımı

Generic 'update' işlemini kullanabilmek için struct tanımınıza aşağıdaki gibi derive makrolarını eklemelisiniz.

```rust
#[derive(Updateable, UpdateParams)]
#[table_name("users")]
#[update_clause("name, email")]
#[where_clause("id = $")]
pub struct UpdateUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}
```

yukarıdaki gibi bir struct tanımlaması yaptıktan sonra eklemeniz gereken toplam 5 adet bağımlılık söz konusu;

```rust
use parsql::{
    core::Updateable,
    macros::{UpdateParams, Updateable},
    tokio_postgres::UpdateParams,
};
use tokio_postgres::types::ToSql;
```

bunlar mevcut küfede bulunan makro, trait ve generic fonksiyonlardan faydalanmanızı sağlayacak.

Sonrasında aşağıdaki gibi bir kullanım ile 'update' işleminizi gerçekleştirebilirsiniz;

```rust
    let update_user = UpdateUser {
        id: 24025,
        name: String::from("Ali"),
        email: String::from("ali@gmail.com"),
        state: 2,
    };

    let result = update(&db, update_user);
```

hepsi bu kadar.
