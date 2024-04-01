use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, Ident};

use crate::util;

pub fn expand_derive_filter(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    match &ast.data {
        // Struct
        Data::Struct(data) => derive_filter_for_struct(&ast.ident, &ast.attrs, data),
        Data::Enum(data) => derive_filter_for_enum(&ast.ident, &ast.attrs, data),
        _ => panic!("filter can only be implemented for structs and enums"),
    }
}

fn derive_filter_for_struct(
    ident: &Ident,
    _attrs: &Vec<Attribute>,
    data: &DataStruct,
) -> TokenStream {
    let filter_ident = util::concat_idents(ident, &Ident::new("Filter", Span::call_site()));

    let fields = match &data.fields {
        Fields::Named(fields) => fields,
        _ => panic!("filter can only be implemented for structs with named fields"),
    };

    let field_variants_decl = fields.named.iter().filter_map(|field| {
        let skip = util::has_skip_filter_attr(&field.attrs);
        if skip {
            return None;
        }

        let field_ident = field.ident.as_ref().unwrap();
        let field_ident_camel_case = Ident::new(
            &util::snake_case_to_camel_case(field_ident.to_string().as_str()),
            Span::call_site(),
        );
        let field_ty = &field.ty;

        Some(
            quote! { #field_ident_camel_case(<#field_ty as ::laser::filter::Filterable>::Filter), },
        )
    });

    let field_variants_impl = fields.named.iter().filter_map(|field| {
        let skip = util::has_skip_filter_attr(&field.attrs);
        if skip {
            return None;
        }

        let field_ident = field.ident.as_ref().unwrap();
        let field_ident_camel_case = Ident::new(
            &util::snake_case_to_camel_case(field_ident.to_string().as_str()),
            Span::call_site(),
        );

        let flat = util::has_flatten_attr(&field.attrs);
        if flat {
            Some(quote! { #filter_ident::#field_ident_camel_case(filter) => filter.push_to_driver(driver), })
        } else {
            Some(quote! { #filter_ident::#field_ident_camel_case(filter) => filter.push_to_driver(stringify!(#field_ident), driver), })
        }
    });

    let expanded = quote! {
        impl ::laser::filter::Filterable for #ident {
            type Filter = #filter_ident;
        }

        #[derive(::std::clone::Clone, ::std::fmt::Debug, ::async_graphql::OneofObject)]
        #[graphql(rename_fields = "snake_case")]
        pub enum #filter_ident {
            All(Vec<#filter_ident>),
            Any(Vec<#filter_ident>),
            #(#field_variants_decl)*
        }

        impl ::laser::driver::PushPrql for #filter_ident {
            fn push_to_driver(&self, driver: &mut ::laser::driver::Driver) {
                match self {
                    #filter_ident::All(all) => {},
                    #filter_ident::Any(any) => {},
                    #(#field_variants_impl)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn derive_filter_for_enum(ident: &Ident, attrs: &Vec<Attribute>, _data: &DataEnum) -> TokenStream {
    let filter_ident = util::concat_idents(ident, &Ident::new("Filter", Span::call_site()));
    let filter_attrs = util::collect_filter_attrs(attrs);

    if filter_attrs.is_empty() {
        panic!("expected at least one of {}", filter_attrs_str());
    }

    let variants = filter_attrs.iter().map(|attr| match attr.as_str() {
        "==" => quote! { Eq(#ident) },
        "!=" => quote! { Ne(#ident) },
        "<" => quote! { Lt(#ident) },
        "<=" => quote! { Le(#ident) },
        ">" => quote! { Gt(#ident) },
        ">=" => quote! { Ge(#ident) },
        _ => panic!(
            "invalid filter attribute, must be one of {}",
            filter_attrs_str()
        ),
    });

    let match_arms = filter_attrs
        .iter()
        .map(|attr| match attr.as_str() {
            "==" => quote! {
                #filter_ident::Eq(x) => {
                    driver.push(column_name);
                    driver.push(" == ");
                    driver.push_bind(x);
                }
            },
            "!=" => quote! {
                #filter_ident::Ne(x) => {
                    driver.push(column_name);
                    driver.push(" != ");
                    driver.push_bind(x);
                }
            },
            "<" => quote! {
                #filter_ident::Lt(x) => {
                    driver.push(column_name);
                    driver.push(" < ");
                    driver.push_bind(x);
                }
            },
            "<=" => quote! {
                #filter_ident::Le(x) => {
                    driver.push(column_name);
                    driver.push(" <= ");
                    driver.push_bind(x);
                }
            },
            ">" => quote! {
                #filter_ident::Gt(x) => {
                    driver.push(column_name);
                    driver.push(" > ");
                    driver.push_bind(x);
                }
            },
            ">=" => quote! {
                #filter_ident::Ge(x) => {
                    driver.push(column_name);
                    driver.push(" >= ");
                    driver.push_bind(x);
                }
            },
            _ => panic!(
                "invalid filter attribute, must be one of {}",
                filter_attrs_str()
            ),
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        impl ::laser::filter::Filterable for #ident {
            type Filter = #filter_ident;
        }

        #[derive(::std::clone::Clone, ::std::fmt::Debug, ::async_graphql::OneofObject)]
        #[graphql(rename_fields = "snake_case")]
        pub enum #filter_ident {
            #(#variants,)*
        }

        impl #filter_ident {
            pub fn push_to_driver(&self, column_name: &'static str, driver: &mut ::laser::driver::Driver) {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

const fn filter_attrs_str() -> &'static str {
    "'==', '!=', '<', '<=', '>', or '>='"
}
