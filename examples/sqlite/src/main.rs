use parsql::{
    core::{Insertable, Queryable, Updateable},
    macros::{FromRow, Insertable, Queryable, SqlParams, UpdateParams, Updateable},
    sqlite::{get, insert, update, FromRow, SqlParams, UpdateParams},
};
use rusqlite::{types::ToSql, Connection, Row, Result, Error};

#[derive(Insertable, SqlParams)]
#[table_name("users")]
pub struct InsertUser {
    pub name: String,
    pub email: String,
    pub state: i16,
}

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

#[derive(Queryable, FromRow, SqlParams, Debug)]
#[table_name("users")]
#[where_clause("id = $")]
pub struct GetUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}

fn main() {
    let conn = Connection::open("sqlite_db.db3").unwrap();

    let _ = conn.execute_batch("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, email TEXT, state INTEGER);");

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
}
