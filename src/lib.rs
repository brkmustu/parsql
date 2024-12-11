pub use parsql_macros as macros;

pub trait Insertable {
    fn table_name() -> &'static str;
    fn columns() -> &'static [&'static str];
}

pub trait Updateable {
    fn table_name() -> &'static str;
    fn update_clause() -> &'static [&'static str];
    fn condition_columns() -> &'static [&'static str];
    fn where_clause() -> &'static str;
}

pub trait Deleteable {
    fn table_name() -> &'static str;
    fn where_clause() -> &'static str;
}

pub trait Queryable {
    fn table_name() -> &'static str;
    fn select_clause() -> &'static [&'static str];
    fn where_clause() -> &'static str;
}
