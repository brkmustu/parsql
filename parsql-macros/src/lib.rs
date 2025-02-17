use proc_macro::TokenStream;

mod crud_impl;

#[proc_macro_derive(Updateable, attributes(table_name, update_clause, where_clause))]
pub fn derive_updateable(input: TokenStream) -> TokenStream {
    crud_impl::derive_updateable_impl(input)
}

#[proc_macro_derive(Insertable, attributes(table_name))]
pub fn derive_insertable(input: TokenStream) -> TokenStream {
    crud_impl::derive_insertable_impl(input)
}

#[proc_macro_derive(Queryable, attributes(table_name, where_clause))]
pub fn derive_queryable(input: TokenStream) -> TokenStream {
    crud_impl::derive_queryable_impl(input)
}

#[proc_macro_derive(Deleteable, attributes(table_name, where_clause))]
pub fn derive_deletable(input: TokenStream) -> TokenStream {
    crud_impl::derive_deletable_impl(input)
}

#[proc_macro_derive(SqlParams, attributes(where_clause))]
pub fn derive_sql_params(input: TokenStream) -> TokenStream {
    crud_impl::derive_sql_params_impl(input)
}

#[proc_macro_derive(UpdateParams, attributes(update_clause, where_clause))]
pub fn derive_update_params(input: TokenStream) -> TokenStream {
    crud_impl::derive_update_params_impl(input)
}

#[proc_macro_derive(FromRow)]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    if cfg!(feature = "sqlite") {
        crud_impl::derive_from_row_sqlite(input)
    } else {
        crud_impl::derive_from_row_postgres(input)
    }
}
