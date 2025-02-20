use std::default;

use example_parsql_postgres::{
    InsertUser, 
    SelectUser, 
    UpdateUser,
    SelectUserWithPosts
};
use parsql::postgres::{get, get_all, insert, select, update};
use postgres::{Client, NoTls};

fn init_connection() -> Client {
    let mut client = Client::connect(
        "host=localhost user=myuser password=mypassword dbname=sample_db",
        NoTls,
    )
    .expect("Postgresql ile bağlantı aşamasında bir hata oluştu!");

    let _ = client.batch_execute("
    CREATE TABLE IF NOT EXISTS users (
        id SERIAL PRIMARY KEY,
        name TEXT,
        email TEXT,
        state INT
    );

    CREATE TABLE IF NOT EXISTS posts (
        id SERIAL PRIMARY KEY,
        user_id INT,
        content TEXT,
        state INT
    );
");

    // let _ = client.batch_execute(
    //     "CREATE TABLE IF NOT EXISTS users (
    //     id SERIAL PRIMARY KEY,
    //     name VARCHAR(100) NOT NULL,
    //     email VARCHAR(255) NOT NULL,
    //     state INTEGER
    // );",
    // ).expect("Postgresql users tablosu oluşturulurken bir hata oluştu!");

    // let _ = client.batch_execute("CREATE TABLE IF NOT EXISTS posts (id SERIAL PRIMARY KEY, user_id INTEGER, content VARCHAR(255) NOT NULL, state INTEGER);").expect("Postgresql posts tablosu oluşturulurken bir hata oluştu!");

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

    let update_user = UpdateUser {
        id: 24025,
        name: String::from("Ali"),
        email: String::from("ali@gmail.com"),
        state: 2,
    };

    let result = update(&mut db, update_user);

    println!("Update result: {:?}", result);

    let select_user = SelectUser {
        id: 24025,
        name: default::Default::default(),
        email: default::Default::default(),
        state: default::Default::default(),
    };

    let select_result = select(&mut db, select_user, |row| {
        Ok(SelectUser{
            id: row.get(0),
            name: row.get(1),
            email: row.get(2),
            state: row.get(3),
        })
    });

    println!("Select result: {:?}", select_result);

    let select_user_with_posts = SelectUserWithPosts::new(1);

    let get_user_with_posts = get(&mut db, &select_user_with_posts);

    println!("Get user with posts: {:?}", get_user_with_posts);
}