use criterion::{black_box, criterion_group, criterion_main, Criterion};
use parsql_core::Insertable;
use parsql_macros::{Insertable, SqlParams};
use parsql_sqlite::{insert, SqlParams};
use rusqlite::{types::ToSql, Connection};

#[derive(Insertable, SqlParams)]
#[table_name("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}

fn init_connection() -> Connection {
    let conn =
        Connection::open("test_db.db3").expect("Test veritabanı oluşturulurken bir hata oluştu!");

    conn.execute_batch(
        "
        DROP TABLE IF EXISTS users;
        CREATE TABLE IF NOT EXISTS users (
          id integer not null primary key,
          name text null,
          email text null,
          state integer null
        );
    ",
    )
    .expect("tablo oluşturulurken bir hata oluştu!");

    conn
}

fn criterion_benchmark(c: &mut Criterion) {
    let db = init_connection();

    c.bench_function("insert user", |b| {
        b.iter(|| {
            let user = black_box(InsertUser {
                name: "SampleName".to_string(),
                email: "SampleEmail".to_string(),
                state: 1,
            });
            let _ = insert(&db, user);
        })
    });
}

criterion_group! {name = benches; config = Criterion::default().measurement_time(std::time::Duration::from_secs(20)).significance_level(0.1).sample_size(10000); targets = criterion_benchmark}
criterion_main!(benches);
