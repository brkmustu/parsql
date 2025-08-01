//! # parsql-macros
//!
//! Procedural macros for the parsql crate.
//! This crate provides derive macros for SQL query generation and parameter handling.
//!
//! ## Features
//!
//! - Automatic SQL query generation
//! - Secure parameter handling
//! - Support for multiple database systems (PostgreSQL, SQLite)
//! - Type-safe database operations
//!
//! ## Macros
//!
//! - `Updateable`: Generates UPDATE queries
//! - `Insertable`: Generates INSERT queries
//! - `Queryable`: Generates SELECT queries
//! - `Deletable`: Generates DELETE queries
//! - `SqlParams`: Generates parameter handling code
//! - `UpdateParams`: Generates parameter handling code for UPDATE operations
//! - `FromRow`: Generates code for converting database rows to Rust structs

use std::env;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod deletable;
mod from_row;
mod insertable;
mod numbering_test;
mod query_builder;
mod queryable;
mod sql_params;
mod update_params;
mod updateable;
mod utils;

#[path = "tests/param_numbering_tests.rs"]
mod param_numbering_tests;
#[path = "tests/sql_param_counter_tests.rs"]
mod sql_param_counter_tests;

mod implementations;

pub(crate) use query_builder::*;
pub(crate) use utils::*;
/// Derive macro for generating UPDATE queries.
///
/// # Attributes
/// - `table`: The name of the table to update
/// - `where_clause`: The WHERE clause for the UPDATE statement
/// - `update`: The columns to update
#[proc_macro_derive(Updateable, attributes(table, where_clause, update))]
pub fn derive_updateable(input: TokenStream) -> TokenStream {
    // Let's add special checks for secure parameter usage
    updateable::derive_updateable_impl(input)
}

/// Derive macro for generating INSERT queries.
///
/// # Attributes
/// - `table`: The name of the table to insert into
/// - `returning`: The column to return after insert (optional)
#[proc_macro_derive(Insertable, attributes(table, returning, sql_type))]
pub fn derive_insertable(input: TokenStream) -> TokenStream {
    insertable::derive_insertable_impl(input)
}

/// Derive macro for generating SELECT queries.
///
/// # Attributes
/// - `table`: The name of the table to select from
/// - `where_clause`: The WHERE clause for the SELECT statement
/// - `select`: The columns to select (optional)
/// - `join`: JOIN clauses (optional)
/// - `group_by`: GROUP BY clause (optional)
/// - `order_by`: ORDER BY clause (optional)
/// - `having`: HAVING clause (optional)
/// - `limit`: LIMIT clause (optional)
/// - `offset`: OFFSET clause (optional)
/// - `distinct`: Make query return distinct results (optional)
/// - `result_type`: The type to return as query result (optional, defaults to self)
#[proc_macro_derive(
    Queryable,
    attributes(
        table,
        where_clause,
        select,
        join,
        group_by,
        order_by,
        having,
        limit,
        offset,
        distinct,
        result_type
    )
)]
pub fn derive_queryable(input: TokenStream) -> TokenStream {
    queryable::derive_queryable_impl(input)
}

/// Derive macro for generating DELETE queries.
///
/// # Attributes
/// - `table`: The name of the table to delete from
/// - `where_clause`: The WHERE clause for the DELETE statement
#[proc_macro_derive(Deletable, attributes(table, where_clause))]
pub fn derive_deletable(input: TokenStream) -> TokenStream {
    deletable::derive_deletable_impl(input)
}

/// Derive macro for generating SQL parameter handling code.
///
/// # Attributes
/// - `where_clause`: The WHERE clause containing parameter placeholders
#[proc_macro_derive(SqlParams, attributes(where_clause))]
pub fn derive_sql_params(input: TokenStream) -> TokenStream {
    sql_params::derive_sql_params_impl(input)
}

/// Derive macro for generating UPDATE parameter handling code.
///
/// # Attributes
/// - `update`: The columns to update
/// - `where_clause`: The WHERE clause containing parameter placeholders
#[proc_macro_derive(UpdateParams, attributes(update, where_clause))]
pub fn derive_update_params(input: TokenStream) -> TokenStream {
    update_params::derive_update_params_impl(input)
}

/// Derive macro for converting database rows to Rust structs.
///
/// This macro automatically detects the enabled database feature and generates
/// the appropriate implementation for converting database rows to Rust structs.
///
/// # Features
/// - `postgres`: Generate code for PostgreSQL
/// - `tokio-postgres`: Generate code for Tokio PostgreSQL
/// - `deadpool-postgres`: Generate code for Deadpool PostgreSQL
/// - `sqlite`: Generate code for SQLite
///
/// # Usage
/// ```rust
/// #[derive(FromRow)]
/// struct User {
///     id: i32,
///     name: String,
///     email: String,
/// }
/// ```
#[proc_macro_derive(FromRow)]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    from_row::expand_from_row(input)
}

// Geriye dönük uyumluluk için özel veritabanı makroları
#[cfg(feature = "sqlite")]
#[proc_macro_derive(FromRowSqlite)]
pub fn derive_from_row_sqlite(input: TokenStream) -> TokenStream {
    crate::implementations::sqlite::generate_from_row(&parse_macro_input!(input as DeriveInput))
        .into()
}

#[cfg(any(
    feature = "postgres",
    feature = "tokio-postgres",
    feature = "deadpool-postgres"
))]
#[proc_macro_derive(FromRowPostgres)]
pub fn derive_from_row_postgres(input: TokenStream) -> TokenStream {
    crate::implementations::postgres::generate_from_row(&parse_macro_input!(input as DeriveInput))
        .into()
}

// SqlParamCounter ve number_where_clause_params fonksiyonlarını sadece test için dışa aktarıyoruz
#[cfg(test)]
pub(crate) use utils::{number_where_clause_params, SqlParamCounter};

/// Log mesajlarını yazdırmak için yardımcı fonksiyon
pub(crate) fn log_message(message: &str) {
    if let Ok(trace) = env::var("PARSQL_TRACE") {
        if trace == "1" {
            println!("{}", message);
        }
    }
}
