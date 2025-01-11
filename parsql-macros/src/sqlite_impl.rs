use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Sadece struct'ları işler.
    let fields = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            &fields.named
        } else {
            panic!("FromRow yalnızca adlandırılmış alanlara sahip struct'lar için desteklenir.");
        }
    } else {
        panic!("FromRow yalnızca struct'lar için desteklenir.");
    };

    // Alan adlarını ve tiplerini çıkarır.
    let field_initializers = fields.iter().map(|field| {
        let name = &field.ident;
        quote! {
            #name: row.get(stringify!(#name))
        }
    });

    // Kod oluşturma
    let expanded = quote! {
        impl FromRow for #name {
            fn from_row(row: &Row) -> Self {
                Self {
                    #(#field_initializers),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}