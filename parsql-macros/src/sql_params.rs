use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

use crate::{
    extract_fields_from_where_clause, log_message, number_where_clause_params, query_builder,
    SqlParamCounter,
};

pub(crate) fn derive_sql_params_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

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

    // HAVING cümlesi için de parametreleri kontrol et
    let having_clause = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("having"))
        .map(|attr| {
            attr.parse_args::<syn::LitStr>()
                .expect("Expected a string literal for having")
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
            panic!("SqlParams can only be derived for structs with named fields");
        }
    } else {
        panic!("SqlParams can only be derived for structs");
    };

    // where_clause ve having_clause'daki parametreleri belirle
    let mut param_fields = Vec::new();

    // WHERE cümlesindeki alan adlarını bulma
    if let Some(clause) = &where_clause {
        // Boş where_clause durumunu kontrol et
        if !clause.trim().is_empty() && clause != "1=1" {
            let where_fields: Vec<_> = fields
                .iter()
                .filter(|&f| clause.contains(f))
                .cloned()
                .collect();
            param_fields.extend(where_fields);
        }
        // Boş where_clause veya "1=1" durumunda parametre ekleme
    }

    // HAVING cümlesindeki alan adlarını bulma
    if let Some(clause) = &having_clause {
        let having_fields: Vec<_> = fields
            .iter()
            .filter(|&f| clause.contains(f))
            .cloned()
            .collect();
        param_fields.extend(having_fields);
    }

    // where_clause yok veya boş değilse ve parametre bulunamadıysa tüm alanları kullan
    // Ancak where_clause boşsa veya "1=1" ise parametre kullanma
    if param_fields.is_empty() && where_clause.is_none() {
        param_fields = fields;
    }

    let field_names: Vec<_> = param_fields
        .iter()
        .map(|f| syn::Ident::new(f, struct_name.span()))
        .collect();

    let expanded = quote! {
        impl SqlParams for #struct_name {
            fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
                vec![#(&self.#field_names as &(dyn ToSql + Sync)),*]
            }
        }
    };

    TokenStream::from(expanded)
}
