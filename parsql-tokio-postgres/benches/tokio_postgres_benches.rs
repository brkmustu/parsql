use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use parsql_macros::{Insertable, SqlParams};
use parsql_tokio_postgres::{insert, SqlParams, SqlQuery};
use tokio_postgres::{types::ToSql, Client, NoTls};

#[derive(Insertable, SqlParams)]
#[table_name("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}

async fn init_connection() -> Client {
    let connection_str = "host=localhost user=myuser password=mypassword dbname=sample_db";
    let (client, connection) = tokio_postgres::connect(connection_str, NoTls)
        .await
        .expect("Postgresql ile bağlantı aşamasında bir hata oluştu!");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let _ = client.batch_execute(
        "CREATE TABLE IF NOT EXISTS users (
        id SERIAL PRIMARY KEY,
        name VARCHAR(100) NOT NULL,
        email VARCHAR(255) NOT NULL,
        state INTEGER
    );",
    );

    client
}

async fn do_parsql_insert(db: &std::cell::RefCell<Client>) {
    let insert_user = InsertUser {
        name: "Ali".to_string(),
        email: "ali@veli".to_string(),
        state: 1,
    };
    let _ = insert(&db.borrow_mut(), insert_user).await;
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let db = std::cell::RefCell::new(rt.block_on(init_connection()));

    c.bench_with_input(
        BenchmarkId::new("tokio postgres", "insert user"),
        &db,
        move |b, db_ref| {
            // Insert a call to `to_async` to convert the bencher to async mode.
            // The timing loops are the same as with the normal bencher.
            b.to_async(&rt).iter(|| do_parsql_insert(db_ref));
        },
    );
}

criterion_group! {name = benches; config = Criterion::default().measurement_time(std::time::Duration::from_secs(20)).significance_level(0.1).sample_size(10000); targets = criterion_benchmark}
criterion_main!(benches);
