use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn expand_derive_type(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let expanded_derive_type =
        sqlx_macros_core::derives::expand_derive_type_encode_decode(&ast).unwrap();

    TokenStream::from(quote! {
        #expanded_derive_type

        impl #impl_generics ::lsor::driver::PushPrql for #ident #ty_generics #where_clause {
            fn push_to_driver(&self, driver: &mut ::lsor::driver::Driver) {
                driver.push_bind(self);
            }
        }
    })
}
