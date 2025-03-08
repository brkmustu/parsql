use proc_macro::TokenStream;

mod crud_impl;

#[proc_macro_derive(Updateable, attributes(table, where_clause, update))]
pub fn derive_updateable(input: TokenStream) -> TokenStream {
    // Güvenli parametre kullanımı için özel kontroller ekleyelim
    crud_impl::derive_updateable_impl(input)
}

#[proc_macro_derive(Insertable, attributes(table))]
pub fn derive_insertable(input: TokenStream) -> TokenStream {
    crud_impl::derive_insertable_impl(input)
}

#[proc_macro_derive(Queryable, attributes(table, where_clause, select, join, group_by, order_by, having))]
pub fn derive_queryable(input: TokenStream) -> TokenStream {
    crud_impl::derive_queryable_impl(input)
}

#[proc_macro_derive(Deleteable, attributes(table, where_clause))]
pub fn derive_deletable(input: TokenStream) -> TokenStream {
    crud_impl::derive_deletable_impl(input)
}

#[proc_macro_derive(SqlParams, attributes(where_clause))]
pub fn derive_sql_params(input: TokenStream) -> TokenStream {
    crud_impl::derive_sql_params_impl(input)
}

#[proc_macro_derive(UpdateParams, attributes(update, where_clause))]
pub fn derive_update_params(input: TokenStream) -> TokenStream {
    crud_impl::derive_update_params_impl(input)
}

// Yeni bir güvenlik katmanı ekleyelim
mod security {
    use super::*;
    
    pub(crate) fn validate_sql_params(params: &[String]) -> Result<(), String> {
        for param in params {
            if contains_sql_injection_patterns(param) {
                return Err(format!("Potansiyel SQL injection tespit edildi: {}", param));
            }
        }
        Ok(())
    }

    fn contains_sql_injection_patterns(input: &str) -> bool {
        let patterns = ["--", ";", "DROP", "DELETE", "UPDATE", "INSERT"];
        patterns.iter().any(|pattern| input.contains(pattern))
    }
}

// FromRow türetmesi için güvenlik kontrolleri
#[proc_macro_derive(FromRow)]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    #[cfg(feature = "sqlite")]
    {
        return crud_impl::derive_from_row_sqlite(input);
    }
    #[cfg(any(feature = "postgres", feature = "tokio-postgres", feature = "deadpool-postgres"))]
    {
        return crud_impl::derive_from_row_postgres(input);
    }
}
