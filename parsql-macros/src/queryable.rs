use crate::{log_message, number_where_clause_params, query_builder, SqlParamCounter};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn derive_queryable_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Extract result_type attribute if present
    let result_type = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("result_type"))
        .map(|attr| {
            attr.parse_args::<syn::LitStr>()
                .expect("Expected a string literal for result_type")
                .value()
        });

    // If result_type is not provided, use the struct itself
    let result_type_ident = if let Some(result_type_str) = result_type {
        syn::Ident::new(&result_type_str, struct_name.span())
    } else {
        struct_name.clone()
    };

    // Table name and column extraction
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

    let fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap().to_string())
                .collect::<Vec<_>>()
        } else {
            panic!("Queryable can only be derived for structs with named fields");
        }
    } else {
        panic!("Queryable can only be derived for structs");
    };

    let joins: Vec<String> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("join"))
        .map(|attr| {
            attr.parse_args::<syn::LitStr>()
                .expect("Expected a string literal for join")
                .value()
        })
        .collect();

    let tables = table.to_string();

    // SQL parametrelerinin numaralandırması için SqlParamCounter kullanıyoruz
    // Bu sayede tüm parametreler her zaman 1'den başlayacak ve tutarlı şekilde artacak
    let mut param_counter = SqlParamCounter::new();

    // WHERE cümlesini numaralandır
    let adjusted_where_clause = where_clause
        .map(|clause| number_where_clause_params(&clause, &mut param_counter))
        .unwrap_or_else(|| "".to_string());

    // Get the optional select attribute
    let select = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("select"))
        .map(|attr| {
            attr.parse_args::<syn::LitStr>()
                .expect("Expected a string literal for select")
                .value()
        });

    // If select is not defined, use all fields
    let select = select.unwrap_or_else(|| {
        fields
            .iter()
            .map(|f| f.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    });

    // Get the optional group_by attribute
    let group_by = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("group_by"))
        .map(|attr| {
            attr.parse_args::<syn::LitStr>()
                .expect("Expected a string literal for group_by")
                .value()
        });

    // Get the optional having attribute
    let having = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("having"))
        .map(|attr| {
            attr.parse_args::<syn::LitStr>()
                .expect("Expected a string literal for having")
                .value()
        });

    // HAVING cümlesi para counter'ın mevcut değerinden devam eder
    // Böylece WHERE cümlesindeki son parametreden sonraki parametreler kullanılır
    let adjusted_having_clause = having
        .as_ref()
        .map(|clause| number_where_clause_params(clause, &mut param_counter))
        .unwrap_or_else(|| "".to_string());

    // Get the optional order_by attribute
    let order_by = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("order_by"))
        .map(|attr| {
            attr.parse_args::<syn::LitStr>()
                .expect("Expected a string literal for order_by")
                .value()
        });

    // Get the optional distinct attribute
    let distinct = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("distinct"))
        .is_some();
    
    // Get distinct columns if specified
    let distinct_columns = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("distinct"))
        .and_then(|attr| {
            attr.parse_args::<syn::LitStr>()
                .ok()
                .map(|lit| lit.value())
        });

    let mut builder = query_builder::SafeQueryBuilder::new();

    builder.add_keyword("SELECT");
    
    // Add DISTINCT if present
    if distinct {
        if let Some(columns) = &distinct_columns {
            // DISTINCT ON (columns) - PostgreSQL specific
            builder.add_keyword("DISTINCT ON");
            builder.add_raw(&format!("({})", columns));
        } else {
            // Simple DISTINCT
            builder.add_keyword("DISTINCT");
        }
    }
    
    builder.add_raw(&select);
    builder.add_keyword("FROM");
    builder.add_identifier(&tables);

    // Add join expressions separately and place a space around each one
    for join in joins {
        builder.add_raw(&format!(" {} ", join.trim()));
    }

    if !adjusted_where_clause.is_empty() {
        builder.add_keyword("WHERE");
        builder.add_raw(&adjusted_where_clause);
    }

    // Add GROUP BY clause
    if let Some(group_by_clause) = group_by {
        builder.add_keyword("GROUP BY");
        builder.add_raw(&group_by_clause);
    }

    // HAVING cümlesi
    if let Some(_) = having {
        builder.add_keyword("HAVING");
        builder.add_raw(&adjusted_having_clause);
    }

    // Add ORDER BY clause
    if let Some(order_by_clause) = order_by {
        builder.add_keyword("ORDER BY");
        builder.add_raw(&order_by_clause);
    }

    // Add LIMIT clause
    let limit = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("limit"))
        .map(|attr| {
            attr.parse_args::<syn::LitInt>()
                .expect("Expected an integer literal for limit")
                .base10_parse::<u64>()
                .expect("Failed to parse limit value as an integer")
        });

    if let Some(limit_value) = limit {
        builder.add_keyword("LIMIT");
        builder.add_raw(&limit_value.to_string());
    }

    // Add OFFSET clause
    let offset = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("offset"))
        .map(|attr| {
            attr.parse_args::<syn::LitInt>()
                .expect("Expected an integer literal for offset")
                .base10_parse::<u64>()
                .expect("Failed to parse offset value as an integer")
        });

    if let Some(offset_value) = offset {
        builder.add_keyword("OFFSET");
        builder.add_raw(&offset_value.to_string());
    }

    let safe_query = builder.build();

    // Log mesajlarını PARSQL_TRACE kontrolü ile yazdır
    log_message(&format!("Generated SQL Query: {}", safe_query));
    log_message(&format!("Total param count: {}", param_counter.count()));

    // Generate SqlQuery implementation with generic type parameter
    let expanded = quote! {
        impl SqlQuery<#result_type_ident> for #struct_name {
            fn query() -> String {
                #safe_query.to_string()
            }
        }
    };

    TokenStream::from(expanded)
}
