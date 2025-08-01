use crate::query_builder;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Implements the Insertable derive macro.
pub(crate) fn derive_insertable_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Extract table name and columns
    let table = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("table"))
        .expect("Missing `#[table = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for table name")
        .value();

    // Extract returning column if specified
    let returning_column = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("returning"))
        .map(|attr| {
            attr.parse_args::<syn::LitStr>()
                .expect("Expected a string literal for returning column")
                .value()
        });

    let fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap().to_string())
                .collect::<Vec<_>>()
        } else {
            panic!("Insertable can only be derived for structs with named fields");
        }
    } else {
        panic!("Insertable can only be derived for structs");
    };

    let column_names = fields.iter().map(|f| f.as_str()).collect::<Vec<_>>();

    // Create placeholders for SQL parameters
    let placeholders: Vec<String> = if cfg!(any(
        feature = "postgres",
        feature = "tokio-postgres",
        feature = "deadpool-postgres"
    )) {
        // PostgreSQL uses numbered placeholders ($1, $2, ...)
        (1..=fields.len()).map(|i| format!("${}", i)).collect()
    } else {
        // SQLite uses ? placeholders
        (0..fields.len()).map(|_| "?".to_string()).collect()
    };

    let mut builder = query_builder::SafeQueryBuilder::new();

    builder.add_keyword("INSERT INTO");
    builder.add_identifier(&table);
    builder.add_raw("(");
    builder.add_comma_list(&column_names);
    builder.add_raw(")");
    builder.add_keyword("VALUES");
    builder.add_raw("(");
    builder.add_raw(&placeholders.join(", "));
    builder.add_raw(")");

    // Add RETURNING clause if specified
    if let Some(returning_col) = returning_column {
        builder.add_keyword("RETURNING");
        builder.add_identifier(&returning_col);
    }

    let safe_query = builder.build();

    // Log the generated SQL if tracing is enabled
    if std::env::var("PARSQL_TRACE").unwrap_or_default() == "1" {
        println!("[PARSQL-MACROS] Generated INSERT SQL: {}", safe_query);
    }

    let expanded = quote! {
        impl SqlCommand for #struct_name {
            fn query() -> String {
                #safe_query.to_string()
            }
        }
    };

    TokenStream::from(expanded)
}
