use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Attribute, LitStr};

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

// Yeni bir güvenlik modülü ekleyelim
mod query_builder {
    pub(crate) struct SafeQueryBuilder {
        pub query: String,
        pub params: Vec<String>,
        pub param_count: usize,
    }

    impl SafeQueryBuilder {
        pub fn new() -> Self {
            Self {
                query: String::new(),
                params: Vec::new(),
                param_count: 0,
            }
        }

        pub fn build(self) -> String {
            self.query.trim().to_string()
        }

        pub fn add_keyword(&mut self, keyword: &str) {
            if !self.query.is_empty() {
                self.query.push_str(" ");
            }
            self.query.push_str(keyword);
        }

        pub fn add_identifier(&mut self, ident: &str) {
            if !self.query.is_empty() {
                self.query.push_str(" ");
            }
            let safe_ident = ident
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>();
            self.query.push_str(&safe_ident);
        }

        pub fn add_comma_list(&mut self, items: &[&str]) {
            let safe_items: Vec<String> = items
                .iter()
                .map(|item| {
                    item.chars()
                        .filter(|c| c.is_alphanumeric() || *c == '_')
                        .collect()
                })
                .collect();
            self.query.push_str(&safe_items.join(", "));
        }

        pub fn add_raw(&mut self, text: &str) {
            if !self.query.is_empty() && !text.starts_with(',') {
                self.query.push_str(" ");
            }
            self.query.push_str(text);
        }
    }
}

pub(crate) fn derive_updateable_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Extracting `table` attribute
    let table = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("table"))
        .expect("Missing `#[table = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for `table`")
        .value();

    // Extracting `columns` attribute
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

    let mut builder = query_builder::SafeQueryBuilder::new();
    
    builder.add_keyword("UPDATE");
    builder.add_identifier(&table);
    builder.add_keyword("SET");

    // SET ifadelerini güvenli şekilde oluştur
    let update_statements: Vec<String> = column_order
        .iter()
        .enumerate()
        .map(|(i, col)| {
            let safe_col = col.chars()
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

    let expanded = quote! {
        impl SqlQuery for #struct_name {
            fn query() -> String {
                #safe_query.to_string()
            }
        }
    };

    TokenStream::from(expanded)
}

pub(crate) fn derive_insertable_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Table name and column extraction
    let table = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("table"))
        .expect("Missing `#[table = \"...\"]` attribute")
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

    let column_names = fields.iter().map(|f| f.as_str()).collect::<Vec<_>>();

    // placeholders'ı Vec<String> olarak oluşturalım, String olarak değil
    let placeholders: Vec<String> = (1..=column_names.len())
        .map(|i| format!("${}", i))
        .collect();

    let mut builder = query_builder::SafeQueryBuilder::new();
    
    builder.add_keyword("INSERT INTO");
    builder.add_identifier(&table);
    builder.add_keyword("(");
    builder.add_comma_list(&column_names);
    builder.add_keyword(")");
    builder.add_keyword("VALUES");
    builder.add_keyword("(");
    
    let placeholder_str = placeholders.join(", ");
    builder.query.push_str(&placeholder_str);
    
    builder.add_keyword(")");

    let safe_query = builder.build();

    let expanded = quote! {
        impl SqlQuery for #struct_name {
            fn query() -> String {
                #safe_query.to_string()
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn derive_queryable_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Table name and column extraction
    let table = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("table"))
        .expect("Missing `#[table = \"...\"]` attribute")
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

    // Opsiyonel select özniteliğini al
    let select = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("select"))
        .map(|attr| {
            attr.parse_args::<syn::LitStr>()
                .expect("Expected a string literal for select")
                .value()
        });

    // Eğer select tanımlanmamışsa, tüm alanları kullan
    let select = select.unwrap_or_else(|| {
        fields
            .iter()
            .map(|f| f.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    });

    let mut builder = query_builder::SafeQueryBuilder::new();
    
    builder.add_keyword("SELECT");
    builder.add_raw(&select);
    builder.add_keyword("FROM");
    builder.add_identifier(&tables);
    
    // Join ifadelerini ayrı ayrı ekleyelim ve her birinin etrafına boşluk koyalım
    for join in joins {
        builder.add_raw(&format!(" {} ", join.trim()));
    }
    
    if !adjusted_where_clause.is_empty() {
        builder.add_keyword("WHERE");
        builder.add_raw(&adjusted_where_clause);
    }

    let safe_query = builder.build();

    let expanded = quote! {
        impl SqlQuery for #struct_name {
            fn query() -> String {
                #safe_query.to_string()
            }
        }
    };

    TokenStream::from(expanded)
}

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

    let where_clause = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("where_clause"))
        .expect("Missing `#[where_clause = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for table name")
        .value();

    let mut count = 1;
    let adjusted_where_clause = where_clause
        .chars()
        .enumerate()
        .map(|(_, c)| {
            if c == '$' {
                format!("${}", count)
            } else {
                c.to_string()
            }
        })
        .collect::<String>();

    let mut builder = query_builder::SafeQueryBuilder::new();
    
    builder.add_keyword("DELETE FROM");
    builder.add_identifier(&table);
    builder.add_keyword("WHERE");
    builder.add_raw(&adjusted_where_clause);  // SafeQueryBuilder otomatik olarak boşluk ekleyecek

    let safe_query = builder.build();

    let expanded = quote! {
        impl SqlQuery for #struct_name {
            fn query() -> String {
                #safe_query.to_string()
            }
        }
    };

    TokenStream::from(expanded)
}

pub(crate) fn derive_sql_params_impl(input: TokenStream) -> TokenStream {
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

pub(crate) fn derive_update_params_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // update özniteliğini al
    let update = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("update"))
        .expect("Missing `#[update = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for update")
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
    let update_fields: Vec<String> = update
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

pub(crate) fn derive_from_row_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields,
            _ => panic!("Sadece named fields destekleniyor"),
        },
        _ => panic!("Sadece struct'lar destekleniyor"),
    };

    let field_names = fields.named.iter().map(|f| f.ident.as_ref().unwrap());

    let field_strings = fields
        .named
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string());

    let expanded = quote! {
        impl FromRow for #name {
            fn from_row(row: &Row) -> Result<Self, Error> {
                Ok(Self {
                    #(#field_names: row.get(#field_strings)?),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

pub fn derive_from_row_postgres(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
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

fn process_join_clause(join_attr: &str) -> String {
    format!(" {} ", join_attr.trim())
}

pub(crate) fn impl_deleteable(ast: &DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;
    let table_name = get_table_name(&ast.attrs);
    let where_clause = get_where_clause(&ast.attrs);
    
    let sql = format!("DELETE FROM {}{}", table_name, where_clause);
    
    let expanded = quote! {
        impl SqlQuery for #struct_name {
            fn query() -> String {
                #sql.to_string()
            }
        }
    };

    TokenStream::from(expanded)
}

fn get_table_name(attrs: &[Attribute]) -> String {
    for attr in attrs {
        if attr.path().is_ident("table") {
            if let Ok(lit) = attr.parse_args::<LitStr>() {
                return lit.value();
            }
        }
    }
    panic!("Missing `#[table = \"...\"]` attribute")
}

fn get_where_clause(attrs: &[Attribute]) -> String {
    for attr in attrs {
        if attr.path().is_ident("where_clause") {
            if let Ok(lit) = attr.parse_args::<LitStr>() {
                // WHERE kelimesi ve koşul arasına boşluk ekliyoruz
                return format!(" WHERE {} ", lit.value());
            }
        }
    }
    panic!("Missing `#[where_clause = \"...\"]` attribute")
}
