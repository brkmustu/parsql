use parsql::{macros::Queryable, Queryable};
use parsql_postgres::SqlParams;

#[derive(Queryable, Debug)]
#[table_name("users")]
#[where_clause("id = $")]
pub struct GetUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}

impl GetUser {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            name: Default::default(),
            email: Default::default(),
            state: Default::default(),
        }
    }
}

#[derive(Queryable, Debug)]
#[table_name("users")]
#[where_clause("email = $")]
pub struct GetAllUsers {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub state: i16,
}

// impl DataMapper<GetUser> for GetUser {
//     fn to_model(row: &postgres::Row) -> Result<GetUser, postgres::Error> {
//         Ok(GetUser {
//             id: row.get(0),
//             name: row.get(1),
//             email: row.get(2),
//             state: row.get(3),
//         })
//     }
// }

// impl DataMapper<GetAllUsers> for GetAllUsers {
//     fn to_model(row: &postgres::Row) -> Result<GetAllUsers, postgres::Error> {
//         Ok(GetAllUsers {
//             id: row.get(0),
//             name: row.get(1),
//             email: row.get(2),
//             state: row.get(3),
//         })
//     }
// }
