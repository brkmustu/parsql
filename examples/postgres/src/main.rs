use example_parsql_postgres::{InsertUser, UpdateUser};
use parsql::postgres::{insert, update};
use postgres::{Client, NoTls};

fn init_connection() -> Client {
    let mut client = Client::connect(
        "host=localhost user=myuser password=mypassword dbname=sample_db",
        NoTls,
    )
    .expect("Postgresql ile bağlantı aşamasında bir hata oluştu!");

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

fn main() {
    let mut db = init_connection();

    let insert_user = InsertUser {
        name: "Ali".to_string(),
        email: "alice@parsql.com".to_string(),
        state: 1,
    };

    let insert_result = insert(&mut db, insert_user);
    println!("Insert result: {:?}", insert_result);

    let update_usert = UpdateUser {
        id: 72306,
        name: String::from("Ali"),
        email: String::from("ali@gmail.com"),
        state: 2,
    };

    let result = update(db, update_usert);
    println!("Update result: {:?}", result);
}
