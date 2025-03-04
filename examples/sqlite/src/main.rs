use parsql::{
    macros::{FromRow, Insertable, Queryable, SqlParams, UpdateParams, Updateable},
    sqlite::{get, insert, update, FromRow, SqlParams, UpdateParams, SqlQuery},
};
use rusqlite::{types::ToSql, Connection, Row, Result, Error};

#[derive(Insertable, SqlParams)]
#[table("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}

#[derive(Updateable, UpdateParams)]
#[table("users")]
#[update("name, email")]
#[where_clause("id = $")]
pub struct UpdateUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}

#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("id = $")]
pub struct GetUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}

#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[where_clause("name = $")]
pub struct GetUserByName {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}

#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table("users")]
#[select("users.state, posts.state as post_state, COUNT(posts.id) as post_count, AVG(CAST(posts.id as REAL)) as avg_post_id")]
#[join("LEFT JOIN posts ON users.id = posts.user_id")]
#[where_clause("users.state > $")]
#[group_by("users.state, posts.state")]
#[having("COUNT(posts.id) > 0 AND AVG(CAST(posts.id as REAL)) > 2")]
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

fn main() {
    let conn = Connection::open("sqlite_db.db3").unwrap();

    // Tabloları oluştur ve örnek veriler ekle
    let _ = conn.execute_batch("
        DROP TABLE IF EXISTS comments;
        DROP TABLE IF EXISTS posts;
        DROP TABLE IF EXISTS users;

        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT, 
            name TEXT NOT NULL, 
            email TEXT NOT NULL, 
            state INTEGER NOT NULL DEFAULT 1
        );
        
        CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            content TEXT NOT NULL,
            state INTEGER NOT NULL DEFAULT 1,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id)
        );
        
        CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            post_id INTEGER NOT NULL,
            content TEXT NOT NULL,
            state INTEGER NOT NULL DEFAULT 1,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (post_id) REFERENCES posts(id)
        );
        
        -- Örnek veriler
        INSERT INTO users (name, email, state) VALUES 
            ('admin', 'admin@example.com', 1),
            ('user1', 'user1@example.com', 1),
            ('user2', 'user2@example.com', 2);
            
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
    ").expect("Tablo oluşturma işlemi başarısız oldu!");

    let insert_usert = InsertUser {
        name: "Ali".to_string(),
        email: "ali@parsql.com".to_string(),
        state: 1,
    };

    let insert_result = insert(&conn, insert_usert);
    println!("işlem başarıyla tamamlandı! Sonuç: {:?}", insert_result);

    let update_user = UpdateUser {
        id: 1,
        name: String::from("Ali"),
        email: String::from("ali@gmail.com"),
        state: 2,
    };

    let update_result = update(&conn, update_user);
    println!("Update result: {:?}", update_result);

    let get_user = GetUser {
        id: 1,
        name: Default::default(),
        email: Default::default(),
        state: Default::default(),
    };

    let get_user_result = get(&conn, get_user);

    println!("get user result: {:?}", get_user_result);

    // SQL Injection Denemesi
    let malicious_name = "' OR '1'='1";
    
    let get_user = GetUserByName {
        id: 0,
        name: malicious_name.to_string(),
        email: String::new(),
        state: 0,
    };

    match get(&conn, get_user) {
        Ok(user) => println!("Bulunan kullanıcı: {:?}", user),
        Err(e) => println!("Hata: {}", e),
    }

    // parsql için isim ile sorgulama örneği
    let get_user = GetUserByName {
        id: 0,
        name: "admin".to_string(),
        email: String::new(),
        state: 0,
    };

    match get(&conn, get_user) {
        Ok(user) => println!("Bulunan kullanıcı: {:?}", user),
        Err(e) => println!("Hata: {}", e),
    }

    // UserPostStatsAdvanced örneği
    let stats_query = UserPostStatsAdvanced::new(0);
    
    match get(&conn, stats_query) {
        Ok(stats) => println!("Gelişmiş kullanıcı-gönderi istatistikleri: {:?}", stats),
        Err(e) => println!("İstatistik sorgulama hatası: {}", e),
    }
    
    // Tüm istatistikleri getirme örneği
    println!("\nTüm gelişmiş kullanıcı-gönderi istatistikleri:");
    
    // let query = UserPostStatsAdvanced::new(0);
    // let sql = query.to_sql_query();
    
    // let mut stmt = conn.prepare(&sql).unwrap();
    // let params = query.to_params();
    
    // let rows = stmt.query_map(params.as_slice(), |row| {
    //     Ok(UserPostStatsAdvanced {
    //         state: row.get(0)?,
    //         post_state: row.get(1)?,
    //         post_count: row.get(2)?,
    //         avg_post_id: row.get(3)?,
    //     })
    // }).unwrap();
    
    // for row in rows {
    //     match row {
    //         Ok(stats) => println!("  {:?}", stats),
    //         Err(e) => println!("  Satır okuma hatası: {}", e),
    //     }
    // }
}
