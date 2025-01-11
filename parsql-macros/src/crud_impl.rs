use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

fn extract_fields_from_where_clause(input: &str) -> Vec<String> {
    let mut fields = Vec::new();

    // Regex deseni: "id = $" veya "name = $" gibi eşleşmeleri yakalar
    let re = Regex::new(r"\b(\w+)\s*=\s*\$").unwrap();

    // Eşleşen alanları bul ve topla
    for cap in re.captures_iter(input) {
        if let Some(field) = cap.get(1) {
            fields.push(field.as_str().to_string());
        }
    }

    fields
}

pub fn derive_updateable_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Extracting `table_name` attribute
    let table_name = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("table_name"))
        .expect("Missing `#[table_name = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for `table_name`")
        .value();

    // Extracting `columns` attribute
    let columns_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("update_clause"))
        .expect("Missing `#[update_clause = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for `columns`")
        .value();

    let column_order: Vec<String> = columns_attr
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // Extracting `where_clause` attribute
    let where_clause = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("where_clause"))
        .expect("Missing `#[where_clause = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for `where_clause`")
        .value();

    // Collecting fields from the struct
    let fields = if let syn::Data::Struct(data) = &input.data {
        if let syn::Fields::Named(fields) = &data.fields {
            fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap().to_string())
                .collect::<Vec<_>>()
        } else {
            panic!("`Updateable` can only be derived for structs with named fields");
        }
    } else {
        panic!("`Updateable` can only be derived for structs");
    };

    // Sorting fields for `updated_columns`
    let sorted_fields: Vec<_> = column_order
        .iter()
        .filter_map(|col| fields.iter().find(|field| *field == col))
        .cloned()
        .collect();

    let column_names = sorted_fields.iter().map(|f| f.as_str());

    // Adjust the where_clause based on the number of updated columns
    let mut count = sorted_fields.len() + 1;
    let adjusted_where_clause = where_clause
        .chars()
        .enumerate()
        .map(|(_, c)| {
            if c == '$' {
                // $ karakterinin yanına numara ekleyelim
                let new_char = format!("${}", count);
                count += 1;
                new_char
            } else {
                // Diğer karakterleri olduğu gibi bırakıyoruz
                c.to_string()
            }
        })
        .collect::<String>();

    let expanded = quote! {
        impl Updateable for #struct_name {
            fn table_name() -> &'static str {
                #table_name
            }

            fn where_clause() -> &'static str {
                #adjusted_where_clause
            }

            fn update_clause() -> &'static [&'static str] {
                &[#(#column_names),*]
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn derive_insertable_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Table name and column extraction
    let table_name = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("table_name"))
        .expect("Missing `#[table_name = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for table name")
        .value();

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

    let column_names = fields.iter().map(|f| f.as_str());

    // Insertable implementation
    let expanded = quote! {
        impl Insertable for #struct_name {
            fn table_name() -> &'static str {
                #table_name
            }

            fn columns() -> &'static [&'static str] {
                &[#(#column_names),*]
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn derive_queryable_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Table name and column extraction
    let table_name = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("table_name"))
        .expect("Missing `#[table_name = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for table name")
        .value();

    // Table name and column extraction
    let where_clause = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("where_clause"))
        .expect("Missing `#[where_clause = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for table name")
        .value();

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

    let mut count = 1;

    let adjusted_where_clause = where_clause
        .chars()
        .enumerate()
        .map(|(_, c)| {
            if c == '$' {
                // $ karakterinin yanına numara ekleyelim
                let new_char = format!("${}", count);
                count += 1;
                new_char
            } else {
                // Diğer karakterleri olduğu gibi bırakıyoruz
                c.to_string()
            }
        })
        .collect::<String>();

    let column_names = fields.iter().map(|f| f.as_str());

    let expanded = quote! {
        impl Queryable for #struct_name {
            fn table_name() -> &'static str {
                #table_name
            }

            fn select_clause() -> &'static [&'static str] {
                &[#(#column_names),*]
            }

            fn where_clause() -> &'static str {
                #adjusted_where_clause
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn derive_deletable_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Table name and column extraction
    let table_name = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("table_name"))
        .expect("Missing `#[table_name = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for table name")
        .value();

    // Table name and column extraction
    let where_clause = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("where_clause"))
        .expect("Missing `#[where_clause = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for table name")
        .value();

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

    let mut count = 1;

    let adjusted_where_clause = where_clause
        .chars()
        .enumerate()
        .map(|(_, c)| {
            if c == '$' {
                // $ karakterinin yanına numara ekleyelim
                let new_char = format!("${}", count);
                count += 1;
                new_char
            } else {
                // Diğer karakterleri olduğu gibi bırakıyoruz
                c.to_string()
            }
        })
        .collect::<String>();

    let expanded = quote! {
        impl Deleteable for #struct_name {
            fn table_name() -> &'static str {
                #table_name
            }

            fn where_clause() -> &'static str {
                #adjusted_where_clause
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn derive_sql_params_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // where_clause özniteliğini opsiyonel olarak al
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
            panic!("SqlParams can only be derived for structs with named fields");
        }
    } else {
        panic!("SqlParams can only be derived for structs");
    };

    // where_clause varsa filtrele, yoksa tüm alanları kullan
    let field_names: Vec<_> = match &where_clause {
        Some(clause) => fields
            .iter()
            .filter(|&f| clause.contains(f))
            .map(|f| syn::Ident::new(f, struct_name.span()))
            .collect(),
        None => fields
            .iter()
            .map(|f| syn::Ident::new(f, struct_name.span()))
            .collect(),
    };

    let expanded = quote! {
        impl SqlParams for #struct_name {
            fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
                vec![#(&self.#field_names as &(dyn ToSql + Sync)),*]
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn derive_update_params_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // update_clause özniteliğini al
    let update_clause = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("update_clause"))
        .expect("Missing `#[update_clause = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for update_clause")
        .value();

    // where_clause özniteliğini al
    let where_clause = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("where_clause"))
        .expect("Missing `#[where_clause = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for where_clause")
        .value();

    let fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap().to_string())
                .collect::<Vec<_>>()
        } else {
            panic!("UpdateParams can only be derived for structs with named fields");
        }
    } else {
        panic!("UpdateParams can only be derived for structs");
    };

    // Güncelleme için kullanılacak alanları al
    let update_fields: Vec<String> = update_clause
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // Where clause için kullanılacak alanları al
    let condition_fields = extract_fields_from_where_clause(&where_clause);

    // Alan isimlerini oluştur
    let update_field_names: Vec<_> = update_fields
        .iter()
        .filter_map(|col| fields.iter().find(|field| *field == col))
        .map(|f| syn::Ident::new(f, struct_name.span()))
        .collect();

    let condition_field_names: Vec<_> = condition_fields
        .iter()
        .filter_map(|col| fields.iter().find(|field| *field == col))
        .map(|f| syn::Ident::new(f, struct_name.span()))
        .collect();

    let expanded = quote! {
        impl UpdateParams for #struct_name {
            fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
                let update_values: Vec<&(dyn ToSql + Sync)> = vec![#(&self.#update_field_names as &(dyn ToSql + Sync)),*];
                let condition_values: Vec<&(dyn ToSql + Sync)> = vec![#(&self.#condition_field_names as &(dyn ToSql + Sync)),*];

                [update_values, condition_values].concat()
            }
        }
    };

    TokenStream::from(expanded)
}

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