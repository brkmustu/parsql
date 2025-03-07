use ex_parsql_tokio_pg::{
    delete_sample::DeleteUser, get_sample::{GetUser, SelectUserWithPosts, UserPostStats, UserStateStats, UserStateStatsFiltered, UserPostStatsAdvanced}, insert_sample::InsertUser,
    update_sample::UpdateUser,
};
use parsql::tokio_postgres::{delete, get, get_all, insert, update};
use postgres::NoTls;

#[tokio::main]
async fn main() {
    // Set PARSQL_TRACE environment variable to log SQL queries
    std::env::set_var("PARSQL_TRACE", "1");

    // PostgreSQL database connection information
    let connection_str = "host=localhost user=myuser password=mypassword dbname=sample_db";
    let (client, connection) = tokio_postgres::connect(connection_str, NoTls)
        .await
        .unwrap();

    // Run the connection in the background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Create tables and insert sample data
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

    -- Sample data
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
    ).await.expect("Table creation failed!");

    // ------------------------------------------------------------
    // INSERT example: Add a new user
    // ------------------------------------------------------------
    // InsertUser struct is marked with #[derive(Insertable, SqlParams)]
    // and #[table("users")] attribute to specify the table
    let insert_user = InsertUser {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
        state: 1_i16,
    };

    // Use insert function to add the new user to the database
    // On success, returns the number of affected rows (1)
    let insert_result = insert(&client, insert_user).await;

    println!("Insert result: {:?}", insert_result);

    // ------------------------------------------------------------
    // UPDATE example: Update an existing user
    // ------------------------------------------------------------
    // UpdateUser struct is marked with #[derive(Updateable, UpdateParams)],
    // #[table("users")] to specify the table,
    // #[update("name, email")] to specify that only name and email fields should be updated,
    // #[where_clause("id = $")] to define the update condition
    let update_user = UpdateUser {
        id: 1,
        name: String::from("John"),
        email: String::from("john@example.com"),
        state: 2,
    };

    // Use update function to update the existing user
    // On success, returns true
    let update_result = update(&client, update_user).await;

    println!("Update result: {:?}", update_result);

    // ------------------------------------------------------------
    // DELETE example: Delete a user
    // ------------------------------------------------------------
    // DeleteUser struct is marked with #[derive(Deleteable, Debug, SqlParams)],
    // #[table("users")] to specify the table,
    // #[where_clause("id = $")] to define the delete condition
    let delete_user = DeleteUser { id: 6 };
    
    // Use delete function to delete the user from the database
    // On success, returns the number of deleted rows
    let delete_result = delete(&client, delete_user).await;

    println!("Delete result: {:?}", delete_result);

    // ------------------------------------------------------------
    // GET example: Query a single user by ID
    // ------------------------------------------------------------
    // GetUser struct is marked with #[derive(Queryable, SqlParams, FromRow, Debug)],
    // #[table("users")] to specify the table,
    // #[where_clause("id = $")] to define the query condition
    let get_user = GetUser::new(1_i32);
    
    // Use get function to retrieve user with ID 1
    // On success, returns the populated GetUser object
    let get_result = get(&client, &get_user).await;
    println!("Get result: {:?}", get_result);

    // ------------------------------------------------------------
    // GET_ALL example: Query a user with related posts and comments
    // ------------------------------------------------------------
    // SelectUserWithPosts struct is marked for a complex query,
    // with JOIN statements, columns to select, and conditions defined with special attributes
    let select_user_with_posts = SelectUserWithPosts::new(1_i32);

    // Use get_all function to retrieve user with ID 1 along with all posts and comments
    // Returns a SelectUserWithPosts object for each post and comment (Cartesian product of related data)
    let get_user_with_posts = get_all(&client, &select_user_with_posts).await;

    println!("Get user with posts: {:?}", get_user_with_posts);

    // ------------------------------------------------------------
    // Advanced query examples: GROUP BY, ORDER BY, HAVING
    // ------------------------------------------------------------
    
    // Example 1: User statistics by state
    // Using GROUP BY and COUNT(*)
    let user_state_stats = get_all(&client, &UserStateStats::new(0)).await;
    println!("User state stats: {:?}", user_state_stats);

    // Example 2: User and post statistics by state
    // Using JOIN, GROUP BY and COUNT
    let user_post_stats = get_all(&client, &UserPostStats::new(0)).await;
    println!("User post stats: {:?}", user_post_stats);

    // Example 3: User state statistics filtered with HAVING
    // Using GROUP BY, COUNT and HAVING
    let user_state_stats_filtered = get_all(&client, &UserStateStatsFiltered::new(0)).await;
    println!("User state stats (filtered with HAVING): {:?}", user_state_stats_filtered);

    // Example 4: Advanced user post statistics
    // Using JOIN, GROUP BY, COUNT, AVG and HAVING
    let user_post_stats_advanced = get_all(&client, &UserPostStatsAdvanced::new(0)).await;
    println!("User post stats (advanced with HAVING): {:?}", user_post_stats_advanced);
}
