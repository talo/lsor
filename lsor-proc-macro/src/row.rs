use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_quote, Data, DeriveInput, Fields, GenericParam, Lifetime, LifetimeDef, WherePredicate,
};

use crate::util;

pub fn expand_derive_row(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let ident = &ast.ident;

    if util::has_json_attr(&ast.attrs) {
        return expand_derive_json_row(ast);
    }

    // get the fields from this struct
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

        let json = util::has_json_attr(&field.attrs);

        let flat = util::has_flatten_attr(&field.attrs);
        if flat {
            quote! { #field_ident: <_>::from_row(row)?, }
        } else if json {
            quote! { #field_ident: row.try_get::<::sqlx::types::Json<_>, _>(stringify!(#field_ident))?.0, }
            // quote! { #field_ident: row.try_get(stringify!(#field_ident))?, }
        } else {
            quote! { #field_ident: row.try_get(stringify!(#field_ident))?, }
        }
    });

    let num_filtered_fields = fields
        .iter()
        .filter(|field| !util::has_skip_attr(&field.attrs))
        .count();

    // expand the implementation of Row::column_names
    let column_names_impl = fields.iter().filter(|field| !util::has_skip_attr(&field.attrs)).enumerate().map(|(i, field)| {
        let flat = util::has_flatten_attr(&field.attrs);
        let postfix = if i < num_filtered_fields - 1 {
            quote! { .chain }
        } else {
            quote! { }
        };
        if flat {
            let field_type = &field.ty;
            quote! { (<#field_type as ::lsor::row::Row>::column_names()) #postfix }
        } else {
            let field_ident = field.ident.as_ref().unwrap();
            let field_pk = util::has_pk_attr(&field.attrs);
            quote! { (Some((::lsor::column::col(stringify!(#field_ident)), #field_pk)).into_iter()) #postfix }
        }
    });

    // expand the implementation of Row::column_values
    let push_column_values_impl = fields
        .iter()
        .filter(|field| !util::has_skip_attr(&field.attrs))
        .enumerate()
        .map(|(i, field)| {
            let postfix = if i < num_filtered_fields - 1 {
                quote! { driver.push(", "); }
            } else {
                quote! {}
            };

            let field_ident = field.ident.as_ref().unwrap();

            let json = util::has_json_attr(&field.attrs);

            if json {
                quote! { driver.push_bind(::sqlx::types::Json(&self.#field_ident)); #postfix }
            } else {
                let flat = util::has_flatten_attr(&field.attrs);
                if flat {
                    quote! { self.#field_ident.push_column_values(driver); #postfix }
                } else {
                    quote! { self.#field_ident.push_to_driver(driver); #postfix }
                }
            }
        });

    // capture the generics before we modify them with the new liftime
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let impl_table_trait = util::collect_table_attr(&ast.attrs).map(|table_name| {
        quote! {
            impl #impl_generics ::lsor::table::Table for #ident #ty_generics #where_clause {
                fn table_name() -> ::lsor::table::TableName {
                    ::lsor::table::table(#table_name)
                }
            }
        }
    });

    let impl_row_trait = quote! {
        impl #impl_generics ::lsor::row::Row for #ident #ty_generics #where_clause {
            fn column_names() -> impl ::std::iter::Iterator<Item = (::lsor::column::ColumnName, bool)> {
                use ::lsor::row::Row;

                #(#column_names_impl)*
            }

            fn push_column_values(&self, driver: &mut ::lsor::driver::Driver) {
                use ::lsor::driver::PushPrql;
                use ::lsor::row::Row;

                #(#push_column_values_impl)*
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

pub fn expand_derive_json_row(mut ast: DeriveInput) -> TokenStream {
    let ident = &ast.ident;

    // modify generic parameters to make sure they are `Send`
    let mut needs_where_clause = false;
    for param in ast.generics.params.iter() {
        if let GenericParam::Type(..) = param {
            needs_where_clause = true;
        }
    }
    if needs_where_clause {
        ast.generics.make_where_clause();
    }
    for param in ast.generics.params.iter() {
        if let GenericParam::Type(type_param) = param {
            let ident = &type_param.ident;
            let predicate: WherePredicate = parse_quote!(#ident: ::std::marker::Sync);
            ast.generics
                .where_clause
                .as_mut()
                .unwrap()
                .predicates
                .push(predicate);
        }
    }

    // capture the generics before we modify them with the new liftime
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // introduce the new lifteime that is needed for the FromRow trait
    let mut generics = ast.generics.clone();
    let lifetime = Lifetime::new("'__sqlx__EncodeDecode", Span::call_site());
    generics
        .params
        .insert(0, GenericParam::Lifetime(LifetimeDef::new(lifetime)));

    // re-capture the impl_generics
    let (impl_generics_with_sqlx_lifetime, _ty_generics, _where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::lsor::driver::PushPrql for #ident #ty_generics #where_clause {
            fn push_to_driver(&self, driver: &mut ::lsor::driver::Driver) {
                driver.push_bind(self);
            }
        }

        impl #impl_generics ::sqlx::Type<::sqlx::Postgres> for #ident #ty_generics #where_clause {
            fn type_info() -> <::sqlx::Postgres as ::sqlx::Database>::TypeInfo {
                ::sqlx::types::JsonValue::type_info()
            }
        }

        impl #impl_generics_with_sqlx_lifetime ::sqlx::Encode<'__sqlx__EncodeDecode, ::sqlx::Postgres> for #ident #ty_generics #where_clause {
            fn encode_by_ref(&self, buf: &mut <::sqlx::Postgres as ::sqlx::database::HasArguments<'__sqlx__EncodeDecode>>::ArgumentBuffer) -> ::sqlx::encode::IsNull {
                ::serde_json::to_value(self)
                    .expect("must serialize json")
                    .encode_by_ref(buf)
            }
        }

        impl #impl_generics_with_sqlx_lifetime ::sqlx::Decode<'__sqlx__EncodeDecode, ::sqlx::Postgres> for #ident #ty_generics #where_clause {
            fn decode(
                value: <::sqlx::Postgres as ::sqlx::database::HasValueRef<'__sqlx__EncodeDecode>>::ValueRef,
            ) -> ::std::result::Result<Self, ::sqlx::error::BoxDynError> {
                Ok(::serde_json::from_value(::sqlx::types::JsonValue::decode(
                    value,
                )?)?)
            }
        }

        impl #impl_generics_with_sqlx_lifetime ::sqlx::postgres::PgHasArrayType for #ident #ty_generics #where_clause {
            fn array_type_info() -> ::sqlx::postgres::PgTypeInfo {
                ::sqlx::types::JsonValue::array_type_info()
            }
        }
    };

    TokenStream::from(expanded)
}
