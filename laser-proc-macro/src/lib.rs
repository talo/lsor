use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident};

pub(crate) mod filter;
pub(crate) mod row;
pub(crate) mod sort;
pub(crate) mod util;
pub(crate) mod ty;

#[proc_macro_derive(Filter, attributes(laser))]
pub fn derive_filter(input: TokenStream) -> TokenStream {
    filter::expand_derive_filter(input)
}

#[proc_macro_derive(Row, attributes(laser))]
pub fn derive_row(input: TokenStream) -> TokenStream {
    row::expand_derive_row(input)
}

#[proc_macro_derive(Type, attributes(laser))]
pub fn derive_type(input: TokenStream) -> TokenStream {
    ty::expand_derive_type(input)
}

#[proc_macro_derive(Laser, attributes(laser))]
pub fn derive_laser(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let try_froms = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().map(|field| {
                let field_name = field.ident.clone().unwrap();
                let field_flatten = util::has_flatten_attr(&field.attrs);
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
                .enumerate()
                .map(|(i, field)| {
                    let field_flatten = util::has_flatten_attr(&field.attrs);
                    let postfix = if i <  named.named.len() - 1 {
                        quote! { .chain }
                    } else {
                        quote! { }
                    };
                    if field_flatten {
                        let field_type = &field.ty;
                        quote! { (<#field_type as ::laser::column::Columns>::columns().into_iter()) #postfix }
                    } else {
                        let field_name = field.ident.as_ref().unwrap();
                        let field_pk = util::has_pk_attr(&field.attrs);
                        quote! { (Some((::laser::column::col(stringify!(#field_name)), #field_pk)).into_iter()) #postfix }
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

    let into_column_values = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().enumerate().map(|(i, field)| {
                let field_flatten = util::has_flatten_attr(&field.attrs);
                let field_name = field.ident.clone().unwrap();
                let infix = if i < named.named.len() - 1 {
                    quote! { qb.push(", "); }
                } else {
                    quote! {}
                };
                if field_flatten {
                    quote! { self.#field_name.into_column_values(qb); #infix }
                } else {
                    quote! { self.#field_name.into_sql(qb); #infix }
                }
            }).collect::<Vec<_>>(),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };

    let sort_by_ident = util::concat_idents(name, &Ident::new("SortBy", Span::call_site()));
    let sort_by_variants = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().filter_map(|field| {
                let field_name = field.ident.clone().unwrap();
                let field_ty = &field.ty;
                if util::has_skip_sort_attr(&field.attrs) {
                    None
                } else {
                    let variant_name = Ident::new(&util::snake_case_to_camel_case(&field_name.to_string()), Span::call_site());
                    let field_flatten = util::has_flatten_attr(&field.attrs);
                    if field_flatten {
                        Some(quote! { #variant_name(<#field_ty as ::laser::sort::Sortable>::Sort) })
                    } else {
                        Some(quote! { #variant_name(::laser::ord::Order) })
                    }
                }
            }),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };
    let sort_by_match_arms = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().filter_map(|field| {
                let field_name = field.ident.clone().unwrap();
                if util::has_skip_sort_attr(&field.attrs) {
                    None
                } else {
                    let variant_name = Ident::new(&util::snake_case_to_camel_case(&field_name.to_string()), Span::call_site());
                    let field_flatten = util::has_flatten_attr(&field.attrs);
                    if field_flatten {
                        Some(quote! {                            
                            #sort_by_ident::#variant_name(sort_by) => {
                                use ::laser::ord::ToOrderBy;
                                sort_by.to_order_by()
                            }
                        })
                    } else {
                        Some(quote! {
                            #sort_by_ident::#variant_name(order) => ::laser::ord::OrderBy { expr: ::laser::column::col(stringify!(#field_name)), order: order.clone() },
                        })
                    }
                }
            }).collect::<Vec<_>>(),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };
    let sort_by_cursor_match_arms = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().filter_map(|field| {
                let field_name = field.ident.clone().unwrap();
                let field_ty = &field.ty;
                if util::has_skip_sort_attr(&field.attrs) {
                    None
                } else {
                    let variant_name = Ident::new(&util::snake_case_to_camel_case(&field_name.to_string()), Span::call_site());
                    let field_flatten = util::has_flatten_attr(&field.attrs);
                    if field_flatten {
                        Some(quote! {                            
                            #sort_by_ident::#variant_name(sort_by) => {
                                sort_by.cursor()
                            }
                        })
                    } else {
                        Some(quote! {                            
                            #sort_by_ident::#variant_name(_) => {
                                use ::laser::cursor::Iterable;
                                <#field_ty as ::laser::cursor::Iterable>::cursor()
                            }
                        })
                    }
                }
            }).collect::<Vec<_>>(),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let impl_table = if let Data::Struct(_) = &ast.data {
        if let Some(table_name) = util::collect_table_attr(&ast.attrs) {
            quote! {
                impl #impl_generics ::laser::table::Table for #name #ty_generics #where_clause {
                    fn table() -> ::laser::table::TableName {
                        ::laser::table::table(#table_name)
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

        impl <'r> #impl_generics ::sqlx::FromRow<'r, ::sqlx::postgres::PgRow> for #name #ty_generics #where_clause {
            fn from_row(row: &'r ::sqlx::postgres::PgRow) -> ::sqlx::Result<Self> {
                use ::sqlx::Row;

                Ok(Self {
                    #(#try_froms,)*
                })
            }
        }

        impl #impl_generics ::laser::column::Columns for #name #ty_generics #where_clause {
            fn columns() -> impl ::std::iter::Iterator<Item = (::laser::column::ColumnName, bool)> {                
                #(#flattened_columns)*
            }

            fn into_column_values(self, qb: &mut ::sqlx::QueryBuilder<'_, ::sqlx::Postgres>) {
                use ::laser::sql::IntoSql;

                #(#into_column_values;)*
            }
        }

        #[derive(::std::clone::Clone, ::std::marker::Copy, ::std::fmt::Debug, ::async_graphql::OneofObject)]
        #[graphql(rename_fields = "snake_case")]
        pub enum #sort_by_ident {
            #(#sort_by_variants,)*
        }

        impl #sort_by_ident {
            pub fn cursor(self) -> ::laser::cursor::Cursor {
                match self {
                    #(#sort_by_cursor_match_arms)*
                }
            }
        }

        impl ::laser::sort::Sortable for #name {
            type Sort = #sort_by_ident;
        }
        
        impl ::laser::ord::ToOrderBy for #sort_by_ident {
            type By = ::laser::column::ColumnName;
        
            fn to_order_by(&self) -> ::laser::ord::OrderBy<Self::By> {
                match self {
                    #(#sort_by_match_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}