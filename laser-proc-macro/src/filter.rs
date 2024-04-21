use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, Ident};

use crate::util;

pub fn expand_derive_filter(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    match &ast.data {
        // Struct
        Data::Struct(data) => expand_derive_filter_for_struct(&ast, &ast.attrs, data),
        Data::Enum(data) => expand_derive_filter_for_enum(&ast, &ast.attrs, data),
        _ => panic!("filter can only be implemented for structs and enums"),
    }
}

fn expand_derive_filter_for_struct(
    ast: &DeriveInput,
    attrs: &[Attribute],
    data: &DataStruct,
) -> TokenStream {
    if util::has_json_attr(attrs) {
        return expand_derive_json_filter_for_struct(ast, attrs, data);
    }

    let ident = &ast.ident;
    let filter_ident = util::concat_idents(ident, &Ident::new("Filter", Span::call_site()));
    let table = util::collect_table_attr(attrs);

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
            Some(quote! { #filter_ident::#field_ident_camel_case(filter) => filter.push_to_driver_with_table_name(tn, driver), })
        } else {
            Some(quote! { #filter_ident::#field_ident_camel_case(filter) => {
                filter.push_to_driver(&::laser::table::dot(tn, ::laser::column::col(stringify!(#field_ident))), driver);
            }})
        }
    });

    let push_to_drive_impl = table.map(|table| {
        quote! {
            impl ::laser::driver::PushPrql for #filter_ident {
                fn push_to_driver(&self, driver: &mut ::laser::driver::Driver) {
                    self.push_to_driver_with_table_name(&::laser::table::table(#table), driver);
                }
            }
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

        #push_to_drive_impl

        impl #filter_ident {
            fn push_to_driver_with_table_name(&self, tn: &dyn ::laser::driver::PushPrql, driver: &mut ::laser::driver::Driver) {
                match &self {
                    #filter_ident::All(all) => {
                        let n = all.len();
                        for (i, x) in all.iter().enumerate() {
                            driver.push('(');
                            x.push_to_driver_with_table_name(tn, driver);
                            if i < n - 1 {
                                driver.push(") && ");
                            } else {
                                driver.push(')');
                            }
                        }
                    },
                    #filter_ident::Any(any) => {
                        let n = any.len();
                        for (i, x) in any.iter().enumerate() {
                            driver.push('(');
                            x.push_to_driver_with_table_name(tn, driver);
                            if i < n - 1 {
                                driver.push(") || ");
                            } else {
                                driver.push(')');
                            }
                        }
                    },
                    #(#field_variants_impl)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn expand_derive_json_filter_for_struct(
    ast: &DeriveInput,
    _attrs: &[Attribute],
    data: &DataStruct,
) -> TokenStream {
    let ident = &ast.ident;
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
            Some(quote! { #filter_ident::#field_ident_camel_case(filter) => {
                filter.push_to_driver(&::laser::column::json(lhs).get(stringify!(#field_ident)), driver);
            }})
        }
    });

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::laser::filter::Filterable for #ident #ty_generics #where_clause {
            type Filter = #filter_ident;
        }

        #[derive(::std::clone::Clone, ::std::fmt::Debug, ::async_graphql::OneofObject)]
        #[graphql(rename_fields = "snake_case")]
        pub enum #filter_ident {
            #(#field_variants_decl)*
        }

        impl #filter_ident {
            pub fn push_to_driver(&self, lhs: &dyn ::laser::driver::PushPrql, driver: &mut ::laser::driver::Driver) {
                match &self {
                    #(#field_variants_impl)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn expand_derive_filter_for_enum(
    ast: &DeriveInput,
    attrs: &[Attribute],
    _data: &DataEnum,
) -> TokenStream {
    if util::has_json_attr(attrs) {
        panic!("filter does not support the #[laser(json)] attribute for enums")
    }

    let ident = &ast.ident;
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
                    lhs.push_to_driver(driver);
                    driver.push(" == ");
                    driver.push_bind(x);
                }
            },
            "!=" => quote! {
                #filter_ident::Ne(x) => {
                    lhs.push_to_driver(driver);
                    driver.push(" != ");
                    driver.push_bind(x);
                }
            },
            "<" => quote! {
                #filter_ident::Lt(x) => {
                    lhs.push_to_driver(driver);
                    driver.push(" < ");
                    driver.push_bind(x);
                }
            },
            "<=" => quote! {
                #filter_ident::Le(x) => {
                    lhs.push_to_driver(driver);
                    driver.push(" <= ");
                    driver.push_bind(x);
                }
            },
            ">" => quote! {
                #filter_ident::Gt(x) => {
                    lhs.push_to_driver(driver);
                    driver.push(" > ");
                    driver.push_bind(x);
                }
            },
            ">=" => quote! {
                #filter_ident::Ge(x) => {
                    lhs.push_to_driver(driver);
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
            pub fn push_to_driver(&self, lhs: &dyn ::laser::driver::PushPrql, driver: &mut ::laser::driver::Driver) {
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
