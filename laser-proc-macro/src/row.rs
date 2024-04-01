use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericParam, Lifetime, LifetimeDef};

use crate::util;

pub fn expand_derive_row(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let ident = &ast.ident;

    let fields = match &ast.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => panic!("row can only be implemented for structs with named fields"),
        },
        _ => panic!("row can only be implemented for structs with named fields"),
    };

    // expand the implementation of FromRow<'r, PgRow>
    let from_row_impl = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();

        let skip = util::has_skip_attr(&field.attrs);
        if skip {
            return quote! { #field_ident: ::std::default::Default::default(), };
        }

        let flat = util::has_flatten_attr(&field.attrs);
        if flat {
            quote! { #field_ident: <_>::from_row(row)?, }
        } else {
            quote! { #field_ident: row.try_get(stringify!(#field_ident))?, }
        }
    });

    // expand the implementation of Row::column_names
    let column_names_impl = fields.iter().enumerate().map(|(i, field)| {
        let field_ident = field.ident.as_ref().unwrap();

        let skip = util::has_skip_attr(&field.attrs);
        if skip {
            return quote! { #field_ident: ::std::default::Default::default(), };
        }

        let flat = util::has_flatten_attr(&field.attrs);
        let postfix = if i < fields.len() - 1 {
            quote! { .chain }
        } else {
            quote! { }
        };
        if flat {
            let field_type = &field.ty;
            quote! { (<#field_type as ::laser::row::Row>::column_names()) #postfix }
        } else {
            let field_ident = field.ident.as_ref().unwrap();
            let field_pk = util::has_pk_attr(&field.attrs);
            quote! { (Some((::laser::column::col(stringify!(#field_ident)), #field_pk)).into_iter()) #postfix }
        }
    });

    // expand the implementation of Row::column_values
    let column_values_impl = fields.iter().enumerate().map(|(i, field)| {
        let field_ident = field.ident.as_ref().unwrap();

        let skip = util::has_skip_attr(&field.attrs);
        if skip {
            return quote! { #field_ident: ::std::default::Default::default(), };
        }

        let flat = util::has_flatten_attr(&field.attrs);
        let postfix = if i < fields.len() - 1 {
            quote! { .chain }
        } else {
            quote! {}
        };

        let field_ident = field.ident.as_ref().unwrap();

        if flat {
            quote! { (self.#field_ident.column_values()) #postfix }
        } else {
            let field_pk = util::has_pk_attr(&field.attrs);
            quote! { (Some((&self.#field_ident as &_, #field_pk)).into_iter()) #postfix }
        }
    });

    // capture the generics before we modify them with the new liftime
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let impl_table_trait = util::collect_table_attr(&ast.attrs).map(|table_name| {
        quote! {
            impl #impl_generics ::laser::table::Table for #ident #ty_generics #where_clause {
                fn table_name() -> ::laser::table::TableName {
                    ::laser::table::table(#table_name)
                }
            }
        }
    });

    let impl_row_trait = quote! {
        impl #impl_generics ::laser::row::Row for #ident #ty_generics #where_clause {
            fn column_names() -> impl ::std::iter::Iterator<Item = (::laser::column::ColumnName, bool)> {
                use ::laser::row::Row;

                #(#column_names_impl)*
            }

            fn column_values(&self) -> impl ::std::iter::Iterator<Item = (&dyn ::laser::driver::PushPrql, bool)> {
                use ::laser::row::Row;

                #(#column_values_impl)*
            }
        }
    };

    // introduce the new lifteime that is needed for the FromRow trait
    let mut generics = ast.generics.clone();
    let lifetime = Lifetime::new("'__sqlx__FromRow", Span::call_site());
    generics
        .params
        .insert(0, GenericParam::Lifetime(LifetimeDef::new(lifetime)));

    // re-capture the impl_generics
    let (impl_generics, _ty_generics, _where_clause) = generics.split_for_impl();

    let expanded = quote! {
        #impl_table_trait

        #impl_row_trait

        impl #impl_generics ::sqlx::FromRow<'__sqlx__FromRow, ::sqlx::postgres::PgRow> for #ident #ty_generics #where_clause {
            fn from_row(row: &'__sqlx__FromRow ::sqlx::postgres::PgRow) -> ::sqlx::Result<Self> {
                use ::sqlx::Row;

                Ok(Self {
                    #(#from_row_impl)*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
