use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

use crate::{
    extract_fields_from_where_clause, log_message, number_where_clause_params, query_builder,
    SqlParamCounter,
};

/// Implements the Updateable derive macro.
pub(crate) fn derive_updateable_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Extract table attribute
    let table = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("table"))
        .expect("Missing `#[table = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for `table`")
        .value();

    // Extract columns attribute
    let columns_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("update"))
        .expect("Missing `#[update = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for `columns`")
        .value();

    let column_order: Vec<String> = columns_attr
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // Get the optional where_clause attribute
    let where_clause = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("where_clause"))
        .map(|attr| {
            attr.parse_args::<syn::LitStr>()
                .expect("Expected a string literal for where_clause")
                .value()
        });

    // Collect fields from the struct
    let fields = if let syn::Data::Struct(data) = &input.data {
        if let syn::Fields::Named(fields) = &data.fields {
            fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap().to_string())
                .collect::<Vec<_>>()
        } else {
            panic!("Updateable can only be derived for structs with named fields");
        }
    } else {
        panic!("Updateable can only be derived for structs");
    };

    // Sort fields for `updated_columns`
    let sorted_fields: Vec<_> = column_order
        .iter()
        .filter_map(|col| fields.iter().find(|field| *field == col))
        .cloned()
        .collect();

    // SQL parametrelerinin numaralandırması için SqlParamCounter kullanıyoruz
    let mut param_counter = SqlParamCounter::new();

    // SET deyiminde kullanılan parametreler sayacı başlatır (1, 2, ...)
    // Her update edilen alan için bir parametre kullanılır
    for _ in 0..sorted_fields.len() {
        param_counter.next();
    }

    // Parametre sayacı update alanlarından sonra devam eder
    // WHERE cümlesindeki parametreler SET parametrelerinden sonraki değerleri alır
    let adjusted_where_clause = where_clause
        .map(|clause| number_where_clause_params(&clause, &mut param_counter))
        .unwrap_or_else(|| "".to_string());

    let mut builder = query_builder::SafeQueryBuilder::new();

    builder.add_keyword("UPDATE");
    builder.add_identifier(&table);
    builder.add_keyword("SET");

    // Build SET statements safely
    let update_statements: Vec<String> = column_order
        .iter()
        .enumerate()
        .map(|(i, col)| {
            let safe_col = col
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>();
            format!("{} = ${}", safe_col, i + 1)
        })
        .collect();

    builder.add_raw(&update_statements.join(", "));

    if !adjusted_where_clause.is_empty() {
        builder.add_keyword("WHERE");
        builder.add_raw(&adjusted_where_clause);
    }

    let safe_query = builder.build();

    // Log mesajlarını PARSQL_TRACE kontrolü ile yazdır
    log_message(&format!("Generated UPDATE SQL: {}", safe_query));
    log_message(&format!("Total param count: {}", param_counter.count()));

    let expanded = quote! {
        impl SqlCommand for #struct_name {
            fn query() -> String {
                #safe_query.to_string()
            }
        }
    };

    TokenStream::from(expanded)
}
