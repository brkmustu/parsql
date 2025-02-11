use proc_macro::TokenStream;
use syn::{self, Data, Fields};
use quote::quote;

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
pub fn from_row_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_from_row_macro(&ast)
}

fn impl_from_row_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    
    let fields = match &ast.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => &fields.named,
                _ => panic!("FromRow only supports structs with named fields"),
            }
        },
        _ => panic!("FromRow only supports structs"),
    };

    let field_names = fields.iter().map(|f| &f.ident);
    let field_names_str = fields.iter().map(|f| f.ident.as_ref().unwrap().to_string());

    let gen = quote! {
        impl FromRow for #name {
            fn from_row(row: &Row) -> Result<Self, Error> {
                Ok(Self {
                    #(
                        #field_names: row.try_get(#field_names_str)?,
                    )*
                })
            }
        }
    };
    gen.into()
}
