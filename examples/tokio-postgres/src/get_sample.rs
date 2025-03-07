use parsql::{
    macros::{FromRow, Queryable, SqlParams},
    tokio_postgres::{FromRow, SqlParams, SqlQuery},
};
use tokio_postgres::{types::ToSql, Row, Error};

/// # GetUser
/// 
/// Data model used for querying a user by ID.
/// 
/// ## Attributes
/// - `#[derive(Queryable, SqlParams, FromRow, Debug)]`: Makes this type support query generation, SQL parameter generation,
///    database row to object conversion, and debugging.
/// - `#[table("users")]`: Specifies that this model will be used with the 'users' table.
/// - `#[where_clause("id = $")]`: Specifies that the query will run with the 'WHERE id = ?' condition.
/// 
/// ## Fields
/// - `id`: ID of the user to query
/// - `name`: User's name
/// - `email`: User's email address
/// - `state`: User's status
/// 
/// ## Usage
/// ```rust
/// // Query user with ID 1
/// let get_user = GetUser::new(1_i32);
/// 
/// // Retrieve from database
/// let get_result = get(&client, &get_user).await;
/// println!("Get result: {:?}", get_result);
/// ```
#[derive(Queryable, SqlParams, FromRow, Debug)]
#[table("users")]
#[where_clause("id = $")]
pub struct GetUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub state: i16,
}

impl GetUser {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            name: String::default(),
            email: String::default(),
            state: 0,
        }
    }
}

/// # GetAllUsers
/// 
/// Data model used for querying users by email address.
/// 
/// ## Attributes
/// - `#[derive(Queryable, SqlParams, FromRow, Debug)]`: Makes this type support query generation, SQL parameter generation,
///    database row to object conversion, and debugging.
/// - `#[table("users")]`: Specifies that this model will be used with the 'users' table.
/// - `#[where_clause("email = $")]`: Specifies that the query will run with the 'WHERE email = ?' condition.
/// 
/// ## Usage
/// ```rust
/// // Query users by email address
/// let user_by_email = GetAllUsers {
///     id: 0,
///     name: String::default(),
///     email: "example@example.com".to_string(),
///     state: 0,
/// };
/// 
/// let users = get_all(&client, &user_by_email).await;
/// ```
#[derive(Queryable, SqlParams, FromRow, Debug)]
#[table("users")]
#[where_clause("email = $")]
pub struct GetAllUsers {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub state: i16,
}

/// # SelectUserWithPosts
/// 
/// Complex data model used for querying a user with their posts and comments.
/// 
/// ## Attributes
/// - `#[derive(Queryable, SqlParams, FromRow, Debug)]`: Makes this type support query generation, SQL parameter generation,
///    database row to object conversion, and debugging.
/// - `#[table("users")]`: Specifies that the main table is 'users'.
/// - `#[select("users.id, users.name, users.email...")]`: Specifies the columns to return in the query result.
/// - `#[join("INNER JOIN posts...")]`: Adds posts table with INNER JOIN to the users table.
/// - `#[join("LEFT JOIN comments...")]`: Adds comments table with LEFT JOIN to the posts table.
/// - `#[where_clause("users.id = $")]`: Specifies that the query will run with the 'WHERE users.id = ?' condition.
/// 
/// ## Fields
/// - `id`: User's ID
/// - `name`: User's name
/// - `email`: User's email address
/// - `user_state`: User's status
/// - `post_id`: ID of the related post
/// - `content`: Post content
/// - `post_state`: Post status
/// - `comment`: Comment on the post (if any)
/// 
/// ## Usage
/// ```rust
/// // Query user with ID 1 including their posts and comments
/// let select_user_with_posts = SelectUserWithPosts::new(1_i32);
/// let get_user_with_posts = get_all(&client, &select_user_with_posts).await;
/// 
/// println!("Get user with posts: {:?}", get_user_with_posts);
/// ```
#[derive(Queryable, SqlParams, FromRow, Debug)]
#[table("users")]
#[select("users.id, users.name, users.email, users.state as user_state, posts.id as post_id, posts.content, posts.state as post_state, comments.content as comment")]
#[join("INNER JOIN posts ON users.id = posts.user_id")]
#[join("LEFT JOIN comments ON posts.id = comments.post_id")]
#[where_clause("users.id = $")]
pub struct SelectUserWithPosts {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub user_state: i16,
    pub post_id: i32,
    pub content: String,
    pub post_state: i16,
    pub comment: Option<String>,
}

impl SelectUserWithPosts {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            name: String::default(),
            email: String::default(),
            user_state: 0,
            post_id: 0,
            content: String::default(),
            post_state: 0,
            comment: None,
        }
    }
}

/// # UserStateStats
/// 
/// Data model used for querying user statistics by status.
/// Demonstrates GROUP BY and ORDER BY capabilities.
/// 
/// ## Attributes
/// - `#[derive(Queryable, SqlParams, FromRow, Debug)]`: Makes this type support query generation, SQL parameter generation,
///    database row to object conversion, and debugging.
/// - `#[table("users")]`: Specifies that the main table is 'users'.
/// - `#[select("users.state, COUNT(*) as user_count")]`: Counts users for each state.
/// - `#[where_clause("state > $")]`: Filters states greater than the specified value.
/// - `#[group_by("users.state")]`: Groups results by user state.
/// - `#[order_by("user_count DESC")]`: Orders by user count in descending order.
/// 
/// ## Usage
/// ```rust
/// // Query user state statistics for states greater than 0
/// let user_state_stats = get_all(&client, &UserStateStats::new(0)).await;
/// println!("User state stats: {:?}", user_state_stats);
/// ```
#[derive(Queryable, SqlParams, FromRow, Debug)]
#[table("users")]
#[select("users.state, COUNT(*) as user_count")]
#[where_clause("state > $")]
#[group_by("users.state")]
#[order_by("user_count DESC")]
pub struct UserStateStats {
    pub state: i16,
    pub user_count: i64,
}

impl UserStateStats {
    pub fn new(min_state: i16) -> Self {
        Self {
            state: min_state,
            user_count: 0,
        }
    }
}

/// # UserPostStats
/// 
/// Data model used for querying statistics about users and posts by status.
/// Demonstrates JOIN with GROUP BY and ORDER BY capabilities.
/// 
/// ## Attributes
/// - `#[table("users")]`: Specifies that the main table is 'users'.
/// - `#[select("users.state, posts.state as post_state, COUNT(posts.id) as post_count")]`: 
///    Selects user state, post state, and counts posts.
/// - `#[join("LEFT JOIN posts ON users.id = posts.user_id")]`: Adds posts table with LEFT JOIN to the users table.
/// - `#[where_clause("users.state > $")]`: Filters users with states greater than the specified value.
/// - `#[group_by("users.state, posts.state")]`: Groups results by user state and post state.
/// - `#[order_by("post_count DESC")]`: Orders by post count in descending order.
/// 
/// ## Usage
/// ```rust
/// // Query post statistics for users with states greater than 0
/// let user_post_stats = get_all(&client, &UserPostStats::new(0)).await;
/// println!("User post stats: {:?}", user_post_stats);
/// ```
#[derive(Queryable, SqlParams, FromRow, Debug)]
#[table("users")]
#[select("users.state, posts.state as post_state, COUNT(posts.id) as post_count")]
#[join("LEFT JOIN posts ON users.id = posts.user_id")]
#[where_clause("users.state > $")]
#[group_by("users.state, posts.state")]
#[order_by("post_count DESC")]
pub struct UserPostStats {
    pub state: i16,
    pub post_state: Option<i16>,
    pub post_count: i64,
}

impl UserPostStats {
    pub fn new(min_state: i16) -> Self {
        Self {
            state: min_state,
            post_state: None,
            post_count: 0,
        }
    }
}

/// # UserStateStatsFiltered
/// 
/// User state statistics query with HAVING filter.
/// 
/// ## Attributes
/// - `#[table("users")]`: Specifies that the main table is 'users'.
/// - `#[select("users.state, COUNT(*) as user_count")]`: Counts users for each state.
/// - `#[where_clause("state > $")]`: Filters states greater than the specified value.
/// - `#[group_by("users.state")]`: Groups results by user state.
/// - `#[having("COUNT(*) > 1")]`: Filters groups with count greater than 1.
/// - `#[order_by("user_count DESC")]`: Orders by user count in descending order.
/// 
/// ## Usage
/// ```rust
/// // Query user state statistics with HAVING filter
/// let user_state_stats_filtered = get_all(&client, &UserStateStatsFiltered::new(0)).await;
/// println!("User state stats (filtered with HAVING): {:?}", user_state_stats_filtered);
/// ```
#[derive(Queryable, SqlParams, FromRow, Debug)]
#[table("users")]
#[select("users.state, COUNT(*) as user_count")]
#[where_clause("state > $")]
#[group_by("users.state")]
#[having("COUNT(*) > 1")]
#[order_by("user_count DESC")]
pub struct UserStateStatsFiltered {
    pub state: i16,
    pub user_count: i64,
}

impl UserStateStatsFiltered {
    pub fn new(min_state: i16) -> Self {
        Self {
            state: min_state,
            user_count: 0,
        }
    }
}

/// # UserPostStatsAdvanced
/// 
/// Advanced query model demonstrating JOIN, GROUP BY, and HAVING filtering capabilities.
/// 
/// ## Attributes
/// - `#[table("users")]`: Specifies that the main table is 'users'.
/// - `#[select("users.state, posts.state as post_state, COUNT(posts.id) as post_count, AVG(posts.id)::REAL as avg_post_id")]`: 
///    Selects user state, post state, post count, and average post ID.
/// - `#[join("LEFT JOIN posts ON users.id = posts.user_id")]`: Adds posts table with LEFT JOIN to the users table.
/// - `#[where_clause("users.state > $")]`: Filters users with states greater than the specified value.
/// - `#[group_by("users.state, posts.state")]`: Groups results by user state and post state.
/// - `#[having("COUNT(posts.id) > 0 AND AVG(posts.id) > 2")]`: Filters groups with post count greater than 0 and average post ID greater than 2.
/// - `#[order_by("post_count DESC")]`: Orders by post count in descending order.
/// 
/// ## Usage
/// ```rust
/// // Create advanced filtered query with HAVING and AVG functions
/// let user_post_stats_advanced = get_all(&client, &UserPostStatsAdvanced::new(0)).await;
/// println!("User post stats (advanced with HAVING): {:?}", user_post_stats_advanced);
/// ```
#[derive(Queryable, SqlParams, FromRow, Debug)]
#[table("users")]
#[select("users.state, posts.state as post_state, COUNT(posts.id) as post_count, AVG(posts.id)::REAL as avg_post_id")]
#[join("LEFT JOIN posts ON users.id = posts.user_id")]
#[where_clause("users.state > $")]
#[group_by("users.state, posts.state")]
#[having("COUNT(posts.id) > 0 AND AVG(posts.id) > 2")]
#[order_by("post_count DESC")]
pub struct UserPostStatsAdvanced {
    pub state: i16,
    pub post_state: Option<i16>,
    pub post_count: i64,
    pub avg_post_id: Option<f32>,
}

impl UserPostStatsAdvanced {
    pub fn new(min_state: i16) -> Self {
        Self {
            state: min_state,
            post_state: None,
            post_count: 0,
            avg_post_id: None,
        }
    }
}
