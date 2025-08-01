use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::implementations;

/// Expands the FromRow derive macro based on enabled database features
pub fn expand_from_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut impls = Vec::new();

    // Add PostgreSQL implementation if any PostgreSQL feature is enabled
    #[cfg(any(
        feature = "postgres",
        feature = "tokio-postgres",
        feature = "deadpool-postgres"
    ))]
    {
        impls.push(implementations::postgres::generate_from_row(&input));
    }

    // Add SQLite implementation if SQLite feature is enabled
    #[cfg(feature = "sqlite")]
    {
        impls.push(implementations::sqlite::generate_from_row(&input));
    }

    // If no database features are enabled, generate a placeholder
    if impls.is_empty() {
        panic!("No database features enabled. Please enable at least one of: postgres, tokio-postgres, deadpool-postgres, sqlite");
    }

    let tokens: TokenStream2 = quote! {
        #(#impls)*
    };

    TokenStream::from(tokens)
}
