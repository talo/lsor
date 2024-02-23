use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Meta};

#[proc_macro_derive(Laser, attributes(laser))]
pub fn derive_laser(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let try_froms = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().map(|field| {
                let field_name = field.ident.clone().unwrap();
                let field_flatten = is_flatten(&field.attrs);
                if field_flatten {
                    quote! { #field_name: <_>::from_row(row)? }
                } else {
                    quote! { #field_name: row.try_get(stringify!(#field_name))? }
                }
            }),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };

    let flattened_columns = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named
                .named
                .iter()
                .filter_map(|field| {
                    let field_flatten = is_flatten(&field.attrs);
                    if field_flatten {
                        let field_type = &field.ty;
                        Some(quote! { <#field_type as ::laser::column::Columns>::columns() })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };
    let flattened_columns = if flattened_columns.len() > 0 {
        quote! {
            [
                #(#flattened_columns,)*
            ]
            .into_iter()
            .flatten()
        }
    } else {
        quote! {
            ::std::iter::empty()
        }
    };

    let columns = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().filter_map(|field| {
                let field_name = field.ident.clone().unwrap();
                let field_flatten = is_flatten(&field.attrs);
                if !field_flatten {
                    let field_pk = is_pk(&field.attrs);
                    Some(quote! { (::laser::column::col(stringify!(#field_name)), #field_pk) })
                } else {
                    None
                }
            }),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };

    let flattened_to_values = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().filter_map(|field| {
                let field_flatten = is_flatten(&field.attrs);
                if field_flatten {
                    let field_name = field.ident.clone().unwrap();
                    let field_type = &field.ty;
                    Some(quote! { <#field_type as ::laser::value::ToValues>::to_values(&self.#field_name) })
                } else {
                    None
                }
            }).collect::<Vec<_>>(),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };
    let flattened_to_values = if flattened_to_values.len() > 0 {
        quote! {
            [
                #(#flattened_to_values,)*
            ]
            .into_iter()
            .flatten()
        }
    } else {
        quote! {
            ::std::iter::empty()
        }
    };

    let to_values = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().filter_map(|field| {
                let field_name = field.ident.clone().unwrap();
                let field_flatten = is_flatten(&field.attrs);
                if !field_flatten {
                    Some(quote! { &self.#field_name as &dyn ::laser::sql::ToSql })
                } else {
                    None
                }
            }),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let impl_table = if let Data::Struct(_) = &ast.data {
        if let Some(table_name) = is_table(&ast.attrs) {
            quote! {
                impl #impl_generics ::laser::table::Table for #name #ty_generics #where_clause {
                    type D = &'static str;

                    fn table() -> ::laser::table::TableName<Self::D> {
                        table(#table_name)
                    }
                }
            }
        } else {
            quote! {}
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #impl_table

        impl<'r>  #impl_generics ::sqlx::FromRow<'r, ::sqlx::postgres::PgRow> for #name #ty_generics #where_clause {
            fn from_row(row: &'r ::sqlx::postgres::PgRow) -> ::sqlx::Result<Self> {
                Ok(Self {
                    #(#try_froms,)*
                })
            }
        }

        impl #impl_generics ::laser::column::Columns for #name #ty_generics #where_clause {
            type D = &'static str;

            fn columns() -> impl ::std::iter::Iterator<Item = (::laser::column::ColumnName<Self::D>, bool)> {
                #flattened_columns
                .chain([
                    #(#columns,)*
                ])
            }
        }

        impl #impl_generics ::laser::value::ToValues for #name #ty_generics #where_clause {
            fn to_values(&self) -> impl ::std::iter::Iterator<Item = &dyn ::laser::sql::ToSql> {
                #flattened_to_values
                .chain([
                    #(#to_values,)*
                ])
            }
        }
    };

    TokenStream::from(expanded)
}

fn is_pk(attrs: &Vec<Attribute>) -> bool {
    for attr in attrs {
        if attr.path().is_ident("laser") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    if let Some("pk") = meta_list
                        .tokens
                        .clone()
                        .into_iter()
                        .next()
                        .map(|t| t.to_string())
                        .as_deref()
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    return false;
}

fn is_flatten(attrs: &Vec<Attribute>) -> bool {
    for attr in attrs {
        if attr.path().is_ident("laser") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    if let Some("flatten") = meta_list
                        .tokens
                        .clone()
                        .into_iter()
                        .next()
                        .map(|t| t.to_string())
                        .as_deref()
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    return false;
}

fn is_table(attrs: &Vec<Attribute>) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("laser") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    let mut meta_list_token_iter = meta_list.tokens.clone().into_iter();
                    match (
                        meta_list_token_iter
                            .next()
                            .map(|t| t.to_string())
                            .as_deref(),
                        meta_list_token_iter
                            .next()
                            .map(|t| t.to_string())
                            .as_deref(),
                        meta_list_token_iter.next().map(|t| t.to_string()),
                    ) {
                        (Some("table"), Some("="), Some(table_name))
                            if table_name.starts_with('\"') && table_name.ends_with('\"') =>
                        {
                            return Some(table_name[1..table_name.len() - 1].to_owned());
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
    return None;
}
