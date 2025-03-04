use ex_parsql_tokio_pg::{
    delete_sample::DeleteUser, get_sample::{GetUser, SelectUserWithPosts, UserPostStats, UserStateStats, UserStateStatsFiltered, UserPostStatsAdvanced}, insert_sample::InsertUser,
    update_sample::UpdateUser,
};
use parsql::tokio_postgres::{delete, get, get_all, insert, update};
use postgres::NoTls;

#[tokio::main]
async fn main() {
    std::env::set_var("PARSQL_TRACE", "1");

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

    // Tabloları oluştur ve örnek veriler ekle
    let _ = client.batch_execute(
        "
    DROP TABLE IF EXISTS comments;
    DROP TABLE IF EXISTS posts;
    DROP TABLE IF EXISTS users;

    CREATE TABLE IF NOT EXISTS users (
        id SERIAL PRIMARY KEY,
        name TEXT NOT NULL,
        email TEXT NOT NULL,
        state SMALLINT NOT NULL DEFAULT 1
    );

    CREATE TABLE IF NOT EXISTS posts (
        id SERIAL PRIMARY KEY,
        user_id INT NOT NULL REFERENCES users(id),
        content TEXT NOT NULL,
        state SMALLINT NOT NULL DEFAULT 1,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS comments (
        id SERIAL PRIMARY KEY,
        post_id INT NOT NULL REFERENCES posts(id),
        content TEXT NOT NULL,
        state SMALLINT NOT NULL DEFAULT 1,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );

    -- Örnek veriler
    INSERT INTO users (name, email, state) VALUES 
        ('Admin', 'admin@example.com', 1),
        ('User1', 'user1@example.com', 1),
        ('User2', 'user2@example.com', 2);

    INSERT INTO posts (user_id, content, state) VALUES 
        (1, 'Admin post 1', 1),
        (1, 'Admin post 2', 1),
        (2, 'User1 post 1', 1),
        (3, 'User2 post 1', 2);

    INSERT INTO comments (post_id, content, state) VALUES 
        (1, 'Comment on admin post 1', 1),
        (1, 'Another comment on admin post 1', 1),
        (2, 'Comment on admin post 2', 1),
        (3, 'Comment on User1 post', 1);
    "
    ).await.expect("Tablo oluşturma işlemi başarısız oldu!");

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

    let user_state_stats = get_all(&client, &UserStateStats::new(0)).await;
    println!("User state stats: {:?}", user_state_stats);

    let user_post_stats = get_all(&client, &UserPostStats::new(0)).await;
    println!("User post stats: {:?}", user_post_stats);

    // HAVING ile filtrelenmiş kullanıcı durumu istatistikleri
    let user_state_stats_filtered = get_all(&client, &UserStateStatsFiltered::new(0)).await;
    println!("User state stats (filtered with HAVING): {:?}", user_state_stats_filtered);

    // Gelişmiş kullanıcı gönderi istatistikleri (HAVING ile filtrelenmiş)
    let user_post_stats_advanced = get_all(&client, &UserPostStatsAdvanced::new(0)).await;
    println!("User post stats (advanced with HAVING): {:?}", user_post_stats_advanced);
}
