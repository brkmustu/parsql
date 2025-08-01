use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

use crate::{
    extract_fields_from_where_clause, log_message, number_where_clause_params, query_builder,
    SqlParamCounter,
};

pub(crate) fn derive_deletable_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let table = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("table"))
        .expect("Missing `#[table = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for table name")
        .value();

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

    // SQL parametrelerinin numaralandırması için SqlParamCounter kullanıyoruz
    // Her zaman 1'den başlar
    let mut param_counter = SqlParamCounter::new();

    let adjusted_where_clause = where_clause
        .map(|clause| number_where_clause_params(&clause, &mut param_counter))
        .unwrap_or_else(|| "".to_string());

    let mut builder = query_builder::SafeQueryBuilder::new();

    builder.add_keyword("DELETE FROM");
    builder.add_identifier(&table);
    builder.add_keyword("WHERE");
    builder.add_raw(&adjusted_where_clause); // SafeQueryBuilder will automatically add spaces

    let safe_query = builder.build();

    // Log mesajlarını PARSQL_TRACE kontrolü ile yazdır
    log_message(&format!("Generated DELETE SQL: {}", safe_query));
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
