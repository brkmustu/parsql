use ex_parsql_tokio_pg::{
    delete_sample::DeleteUser, get_sample::{GetUser, SelectUserWithPosts}, insert_sample::InsertUser,
    update_sample::UpdateUser,
};
use parsql::tokio_postgres::{delete, get, get_all, insert, update};
use postgres::NoTls;

#[tokio::main]
async fn main() {
    let connection_str = "host=localhost user=myuser password=mypassword dbname=sample_db";
    let (client, connection) = tokio_postgres::connect(connection_str, NoTls)
        .await
        .unwrap();

    // Bağlantıyı arka planda çalıştır
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let _ = client.batch_execute(
        "
    CREATE TABLE IF NOT EXISTS users (
        id SERIAL PRIMARY KEY,
        name TEXT,
        email TEXT,
        state SMALLINT
    );

    CREATE TABLE IF NOT EXISTS posts (
        id SERIAL PRIMARY KEY,
        user_id INT,
        content TEXT,
        state SMALLINT
    );

    CREATE TABLE IF NOT EXISTS comments (
        id SERIAL PRIMARY KEY,
        post_id INT,
        content TEXT,
        state SMALLINT
    );
",
    );

    let insert_user = InsertUser {
        name: "Ali".to_string(),
        email: "alice@parsql.com".to_string(),
        state: 1_i16,
    };

    let insert_result = insert(&client, insert_user).await;

    println!("Insert result: {:?}", insert_result);

    let update_user = UpdateUser {
        id: 1,
        name: String::from("Ali"),
        email: String::from("ali@gmail.com"),
        state: 2,
    };

    let update_result = update(&client, update_user).await;

    println!("Update result: {:?}", update_result);

    let delete_user = DeleteUser { id: 6 };
    let delete_result = delete(&client, delete_user).await;

    println!("Delete result: {:?}", delete_result);

    let get_user = GetUser::new(1_i32);
    let get_result = get(&client, &get_user).await;
    println!("Get result: {:?}", get_result);

    let select_user_with_posts = SelectUserWithPosts::new(1_i32);

    let get_user_with_posts = get_all(&client, &select_user_with_posts).await;

    println!("Get user with posts: {:?}", get_user_with_posts);
}
