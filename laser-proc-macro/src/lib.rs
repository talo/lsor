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

#[proc_macro_derive(Sort)]
pub fn derive_sort(input: TokenStream) -> TokenStream {
    sort::expand_derive_sort(input)
}

#[proc_macro_derive(Type, attributes(sqlx))]
pub fn derive_type(input: TokenStream) -> TokenStream {
    ty::expand_derive_type(input)
}

#[proc_macro_derive(Laser, attributes(laser))]
pub fn derive_laser(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

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

    let expanded = quote! {
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