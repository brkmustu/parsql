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

use proc_macro::TokenStream;

mod crud_impl;

/// Derive macro for generating UPDATE queries.
/// 
/// # Attributes
/// - `table`: The name of the table to update
/// - `where_clause`: The WHERE clause for the UPDATE statement
/// - `update`: The columns to update
#[proc_macro_derive(Updateable, attributes(table, where_clause, update))]
pub fn derive_updateable(input: TokenStream) -> TokenStream {
    // Let's add special checks for secure parameter usage
    crud_impl::derive_updateable_impl(input)
}

/// Derive macro for generating INSERT queries.
/// 
/// # Attributes
/// - `table`: The name of the table to insert into
#[proc_macro_derive(Insertable, attributes(table))]
pub fn derive_insertable(input: TokenStream) -> TokenStream {
    crud_impl::derive_insertable_impl(input)
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
#[proc_macro_derive(Queryable, attributes(table, where_clause, select, join, group_by, order_by, having))]
pub fn derive_queryable(input: TokenStream) -> TokenStream {
    crud_impl::derive_queryable_impl(input)
}

/// Derive macro for generating DELETE queries.
/// 
/// # Attributes
/// - `table`: The name of the table to delete from
/// - `where_clause`: The WHERE clause for the DELETE statement
#[proc_macro_derive(Deletable, attributes(table, where_clause))]
pub fn derive_deletable(input: TokenStream) -> TokenStream {
    crud_impl::derive_deletable_impl(input)
}

/// Derive macro for generating SQL parameter handling code.
/// 
/// # Attributes
/// - `where_clause`: The WHERE clause containing parameter placeholders
#[proc_macro_derive(SqlParams, attributes(where_clause))]
pub fn derive_sql_params(input: TokenStream) -> TokenStream {
    crud_impl::derive_sql_params_impl(input)
}

/// Derive macro for generating UPDATE parameter handling code.
/// 
/// # Attributes
/// - `update`: The columns to update
/// - `where_clause`: The WHERE clause containing parameter placeholders
#[proc_macro_derive(UpdateParams, attributes(update, where_clause))]
pub fn derive_update_params(input: TokenStream) -> TokenStream {
    crud_impl::derive_update_params_impl(input)
}

/// Derive macro for converting database rows to Rust structs.
/// 
/// This macro generates code for converting database rows to Rust structs based on
/// the enabled database feature (postgres or sqlite).
/// 
/// # Features
/// - `postgres`: Generate code for PostgreSQL
/// - `sqlite`: Generate code for SQLite
#[proc_macro_derive(FromRow)]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    #[cfg(any(feature = "postgres", feature = "tokio-postgres", feature = "deadpool-postgres"))]
    {
        return crud_impl::derive_from_row_postgres(input);
    }
    #[cfg(feature = "sqlite")]
    {
        return crud_impl::derive_from_row_sqlite(input);
    }
    #[cfg(not(any(feature = "postgres", feature = "tokio-postgres", feature = "deadpool-postgres", feature = "sqlite")))]
    {
        panic!("At least one database feature must be enabled (postgres or sqlite)");
    }
}

