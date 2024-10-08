use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::util;

pub fn expand_derive_type(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let expanded_derive_type =
        sqlx_macros_core::derives::expand_derive_type_encode_decode(&ast).unwrap();

    let array_type_name = format!("_{}", util::camel_case_to_snake_case(&ident.to_string()));

    TokenStream::from(quote! {
        #expanded_derive_type

        impl #impl_generics ::lsor::driver::PushPrql for #ident #ty_generics #where_clause {
            fn push_to_driver(&self, driver: &mut ::lsor::driver::Driver) {
                driver.push_bind(self);
            }
        }

        impl #impl_generics ::sqlx::postgres::PgHasArrayType for #ident #ty_generics #where_clause {
            fn array_type_info() -> ::sqlx::postgres::PgTypeInfo {
                ::sqlx::postgres::PgTypeInfo::with_name(#array_type_name)
            }
        }
    })
}
