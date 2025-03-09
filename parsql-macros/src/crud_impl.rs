use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Extracts field names from a WHERE clause.
/// 
/// # Arguments
/// * `input` - The WHERE clause string
/// 
/// # Returns
/// * `Vec<String>` - A vector of field names found in the WHERE clause
fn extract_fields_from_where_clause(input: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let re = Regex::new(r"\b(\w+)\s*=\s*\$").unwrap();
    for cap in re.captures_iter(input) {
        if let Some(field) = cap.get(1) {
            fields.push(field.as_str().to_string());
        }
    }
    fields
}

/// Query builder module for safe SQL query construction
mod query_builder {
    /// A safe query builder that prevents SQL injection
    #[derive(Default)]
    pub(crate) struct SafeQueryBuilder {
        /// The SQL query being built
        pub query: String,
    }

    impl SafeQueryBuilder {
        /// Creates a new empty query builder
        pub fn new() -> Self {
            Self::default()
        }

        /// Builds and returns the final SQL query string
        pub fn build(self) -> String {
            self.query.trim().to_string()
        }

        /// Adds a SQL keyword to the query with proper spacing
        pub fn add_keyword(&mut self, keyword: &str) {
            if !self.query.is_empty() {
                self.query.push(' ');
            }
            self.query.push_str(keyword);
        }

        /// Adds a safe identifier (table name, column name) to the query
        /// 
        /// # Arguments
        /// * `ident` - The identifier to add
        pub fn add_identifier(&mut self, ident: &str) {
            if !self.query.is_empty() {
                self.query.push(' ');
            }
            let safe_ident = ident
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>();
            self.query.push_str(&safe_ident);
        }

        /// Adds a comma-separated list of safe identifiers to the query
        /// 
        /// # Arguments
        /// * `items` - The list of identifiers to add
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

        /// Adds raw text to the query with proper spacing
        /// This should only be used for trusted input or pre-validated strings
        /// 
        /// # Arguments
        /// * `text` - The raw text to add
        pub fn add_raw(&mut self, text: &str) {
            if !self.query.is_empty() && !text.starts_with(',') {
                self.query.push(' ');
            }
            self.query.push_str(text);
        }
    }
}

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

    // Adjust the where_clause based on the number of updated columns
    let mut count = sorted_fields.len() + 1;
    let adjusted_where_clause = where_clause
        .map(|clause| {
            clause.chars()
                .enumerate()
                .map(|(_, c)| {
                    if c == '$' {
                        // Add a number after the $ character
                        let new_char = format!("${}", count);
                        count += 1;
                        new_char
                    } else {
                        // Keep other characters as is
                        c.to_string()
                    }
                })
                .collect::<String>()
        })
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

    // Create placeholders as Vec<String>
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

    let mut count = 1;

    let adjusted_where_clause = where_clause
        .map(|clause| {
            clause.chars()
                .enumerate()
                .map(|(_, c)| {
                    if c == '$' {
                        // Add a number after the $ character
                        let new_char = format!("${}", count);
                        count += 1;
                        new_char
                    } else {
                        // Keep other characters as is
                        c.to_string()
                    }
                })
                .collect::<String>()
        })
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

    let mut builder = query_builder::SafeQueryBuilder::new();
    
    builder.add_keyword("SELECT");
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

    // Add HAVING clause
    if let Some(having_clause) = having {
        builder.add_keyword("HAVING");
        builder.add_raw(&having_clause);
    }

    // Add ORDER BY clause
    if let Some(order_by_clause) = order_by {
        builder.add_keyword("ORDER BY");
        builder.add_raw(&order_by_clause);
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

    let mut count = 1;
    let adjusted_where_clause = where_clause
        .map(|clause| {
            clause.chars()
                .enumerate()
                .map(|(_, c)| {
                    if c == '$' {
                        let new_char = format!("${}", count);
                        count += 1;
                        new_char
                    } else {
                        c.to_string()
                    }
                })
                .collect::<String>()
        })
        .unwrap_or_else(|| "".to_string());

    let mut builder = query_builder::SafeQueryBuilder::new();
    
    builder.add_keyword("DELETE FROM");
    builder.add_identifier(&table);
    builder.add_keyword("WHERE");
    builder.add_raw(&adjusted_where_clause);  // SafeQueryBuilder will automatically add spaces

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

    // Get the update attribute
    let update = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("update"))
        .expect("Missing `#[update = \"...\"]` attribute")
        .parse_args::<syn::LitStr>()
        .expect("Expected a string literal for update")
        .value();

    // Get the where_clause attribute
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

    // Get fields to be used for update
    let update_fields: Vec<String> = update
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // Get fields to be used in the where clause
    let condition_fields = extract_fields_from_where_clause(&where_clause);

    // Create field names
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

/// Implements the FromRow trait for SQLite database
/// 
/// # Arguments
/// * `input` - TokenStream containing the struct definition
/// 
/// # Returns
/// * `TokenStream` - Generated implementation code
#[cfg(feature = "sqlite")]
pub(crate) fn derive_from_row_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
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

/// Implements the FromRow trait for PostgreSQL database
/// 
/// # Arguments
/// * `input` - TokenStream containing the struct definition
/// 
/// # Returns
/// * `TokenStream` - Generated implementation code
pub(crate) fn derive_from_row_postgres(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    
    let fields = match &ast.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("FromRow only supports structs with named fields"),
        },
        _ => panic!("FromRow only supports structs"),
    };

    let field_names = fields.iter().map(|f| &f.ident);
    let field_names_str = fields.iter().map(|f| f.ident.as_ref().unwrap().to_string());

    let gen = quote! {
        impl FromRow for #name {
            fn from_row(row: &Row) -> Result<Self, Error> {
                Ok(Self {
                    #(#field_names: row.try_get(#field_names_str)?),*
                })
            }
        }
    };
    gen.into()
}

