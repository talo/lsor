use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub fn expand_derive_sort(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    // get the fields from this struct
    let fields = match &ast.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => panic!("sort can only be implemented for structs with named fields"),
        },
        _ => panic!("sort can only be implemented for structs with named fields"),
    };

    TokenStream::from(quote! {})
}
