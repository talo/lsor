use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident};

use crate::util;

pub fn expand_derive_sort(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let fields = match &ast.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named,
            _ => panic!("sort can only be implemented for structs with named fields"),
        },
        _ => panic!("sort can only be implemented for structs with named fields"),
    };

    let has_json_attr = util::has_json_attr(&ast.attrs);

    let ident = &ast.ident;
    let sort_ident = util::concat_idents(ident, &Ident::new("Sort", Span::call_site()));

    let mut sort_ident_variant_decls = Vec::new();
    let mut push_to_driver_impls = Vec::new();
    let mut order_impls = Vec::new();
    let mut flip_impls = Vec::new();
    let mut push_to_driver_with_order_impls = Vec::new();
    let mut cursor_impls = Vec::new();

    for field in fields.named.iter() {
        let skip = util::has_skip_sort_attr(&field.attrs);
        if skip {
            continue;
        }

        let field_ident = field.ident.as_ref().unwrap();
        let field_ident_camel_case = Ident::new(
            &util::snake_case_to_camel_case(field_ident.to_string().as_str()),
            Span::call_site(),
        );
        let field_ty = &field.ty;

        let flat = util::has_flatten_attr(&field.attrs);

        sort_ident_variant_decls
            .push(quote! { #field_ident_camel_case(<#field_ty as ::lsor::sort::Sortable>::Sort), });
        if flat {
            push_to_driver_impls.push(quote! {
                #sort_ident::#field_ident_camel_case(sort) => {
                    use ::lsor::driver::PushPrql;

                    sort.push_to_driver(driver);
                },
            });
        } else if has_json_attr {
            push_to_driver_impls.push(quote! {
                #sort_ident::#field_ident_camel_case(sort) => {
                    sort.push_to_driver_with_lhs(&::lsor::column::json(lhs).get(stringify!(#field_ident)), driver);
                },
            });
        } else {
            push_to_driver_impls.push(quote! {
                #sort_ident::#field_ident_camel_case(sort) => {
                    sort.push_to_driver_with_lhs(&::lsor::column::col(stringify!(#field_ident)), driver);
                },
            });
        }
        order_impls.push(quote! {
            #sort_ident::#field_ident_camel_case(sort) => sort.order(),
        });
        flip_impls.push(quote! {
            #sort_ident::#field_ident_camel_case(sort) => #sort_ident::#field_ident_camel_case(sort.flip_as_self()),
        });
        if flat {
            push_to_driver_with_order_impls.push(quote! {
                #sort_ident::#field_ident_camel_case(sort) => {
                    use ::lsor::sort::Sorting;

                    sort.push_to_driver_with_order(driver);
                },
            });
        } else if has_json_attr {
            push_to_driver_with_order_impls.push(quote! {
                #sort_ident::#field_ident_camel_case(sort) => {
                    sort.push_to_driver_with_order_with_lhs(&::lsor::column::json(lhs).get(stringify!(#field_ident)), driver);
                },
            });
        } else {
            push_to_driver_with_order_impls.push(quote! {
                #sort_ident::#field_ident_camel_case(sort) => {
                    sort.push_to_driver_with_order_with_lhs(&::lsor::column::col(stringify!(#field_ident)), driver);
                },
            });
        }
        cursor_impls.push(quote! {
            #sort_ident::#field_ident_camel_case(x) => {
                use ::lsor::cursor::Iterable;
                x.cursor()
            }
        });
    }

    let trait_impls = if !has_json_attr {
        Some(quote! {
            impl ::lsor::driver::PushPrql for #sort_ident {
                fn push_to_driver(&self, driver: &mut ::lsor::driver::Driver) {
                    match &self {
                        #(#push_to_driver_impls)*
                    }
                }
            }

            impl ::lsor::sort::Sorting for #sort_ident {
                fn order(&self) -> ::lsor::sort::Order {
                    match self {
                        #(#order_impls)*
                    }
                }

                fn flip(&self) -> impl ::lsor::sort::Sorting {
                    match self {
                        #(#flip_impls)*
                    }
                }

                fn push_to_driver_with_order(&self, driver: &mut ::lsor::driver::Driver) {
                    match &self {
                        #(#push_to_driver_with_order_impls)*
                    }
                }
            }
        })
    } else {
        None
    };

    let non_trait_order_impl = if has_json_attr {
        Some(quote! {
            pub fn order(&self) -> ::lsor::sort::Order {
                match self {
                    #(#order_impls)*
                }
            }
        })
    } else {
        None
    };
    let non_trait_impls = quote! {
        impl #sort_ident {
            pub fn push_to_driver_with_lhs(&self, lhs: &dyn ::lsor::driver::PushPrql, driver: &mut ::lsor::driver::Driver) {
                match &self {
                    #(#push_to_driver_impls)*
                }
            }

            pub fn push_to_driver_with_order_with_lhs(&self, lhs: &dyn ::lsor::driver::PushPrql, driver: &mut ::lsor::driver::Driver) {
                match &self {
                    #(#push_to_driver_with_order_impls)*
                }
            }

            pub fn flip_as_self(&self) -> #sort_ident {
                match self {
                    #(#flip_impls)*
                }
            }

            #non_trait_order_impl
        }
    };

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::lsor::sort::Sortable for #ident #ty_generics #where_clause {
            type Sort = #sort_ident;
        }

        impl #sort_ident {
            pub fn cursor(&self) -> ::lsor::cursor::Cursor {
                match self {
                    #(#cursor_impls)*
                }
            }
        }

        #[derive(::std::clone::Clone, ::std::fmt::Debug, ::async_graphql::OneofObject)]
        #[graphql(rename_fields = "snake_case")]
        pub enum #sort_ident {
            #(#sort_ident_variant_decls)*
        }

        #non_trait_impls

        #trait_impls
    };

    TokenStream::from(expanded)
}
