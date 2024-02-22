use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub fn derive_row(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let name_as_str = format!("{name}");

    let description = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => {
                let fields = named.named.iter().map(|field| {
                    let field_name = format!("{}", field.ident.clone().unwrap());
                    let field_type = &field.ty;
                    todo!();
                });
                todo!()
            }
            Fields::Unnamed(..) => panic!("Row cannot be derived for structs with unnamed fields"),
            Fields::Unit => panic!("Row cannot be derived for unit-structs"),
        },
        _ => panic!("Row cannot be derived for non-structs"),
    };

    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let generic_type_names = generics
        .params
        .iter()
        .map(|param| match param {
            syn::GenericParam::Type(type_param) => {
                let ident = &type_param.ident;
                quote! { #ident::tname() }
            }
            _ => panic!("Generic type parameters are the only supported generics"),
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        // impl  #impl_generics ::ouroboros::TypeInfo for #name #ty_generics #where_clause {
        //     fn tname() -> ::ouroboros::TypeName {
        //         ::ouroboros::TypeName { n: #name_as_str, g: vec![#(#generic_type_names,)*] }
        //     }

        //     fn t() -> ::ouroboros::Type {
        //         #description
        //     }
        // }
    };

    TokenStream::from(expanded)
}
