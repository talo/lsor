use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Ident, Meta};

#[proc_macro_derive(Laser, attributes(laser))]
pub fn derive_laser(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let try_froms = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().map(|field| {
                let field_name = field.ident.clone().unwrap();
                let field_flatten = is_flatten(&field.attrs);
                if field_flatten {
                    quote! { #field_name: <_>::from_row(row)? }
                } else {
                    quote! { #field_name: row.try_get(stringify!(#field_name))? }
                }
            }),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };

    let flattened_columns = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named
                .named
                .iter()
                .filter_map(|field| {
                    let field_flatten = is_flatten(&field.attrs);
                    if field_flatten {
                        let field_type = &field.ty;
                        Some(quote! { <#field_type as ::laser::column::Columns>::columns() })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };
    let flattened_columns = if flattened_columns.len() > 0 {
        quote! {
            [
                #(#flattened_columns,)*
            ]
            .into_iter()
            .flatten()
        }
    } else {
        quote! {
            ::std::iter::empty()
        }
    };

    let columns = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().filter_map(|field| {
                let field_name = field.ident.clone().unwrap();
                let field_flatten = is_flatten(&field.attrs);
                if !field_flatten {
                    let field_pk = is_pk(&field.attrs);
                    Some(quote! { (::laser::column::col(stringify!(#field_name)), #field_pk) })
                } else {
                    None
                }
            }),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };

    let into_column_values = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().map(|field| {
                let field_flatten = is_flatten(&field.attrs);
                let field_name = field.ident.clone().unwrap();
                if field_flatten {
                    quote! { self.#field_name.into_column_values(qb) }
                } else {
                    quote! { self.#field_name.into_sql(qb) }
                }
            }).collect::<Vec<_>>(),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };

    let sort_by_ident = concatenate_idents(&name, &Ident::new("SortBy", Span::call_site()));
    let sort_by_variants = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().filter_map(|field| {
                let field_name = field.ident.clone().unwrap();
                let field_ty = &field.ty;
                if is_skip_sort_by(&field.attrs) {
                    None
                } else {
                    let variant_name = Ident::new(&snake_case_to_camel_case(&field_name.to_string()), Span::call_site());
                    let field_flatten = is_flatten(&field.attrs);
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
                if is_skip_sort_by(&field.attrs) {
                    None
                } else {
                    let variant_name = Ident::new(&snake_case_to_camel_case(&field_name.to_string()), Span::call_site());
                    let field_flatten = is_flatten(&field.attrs);
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
                if is_skip_sort_by(&field.attrs) {
                    None
                } else {
                    let variant_name = Ident::new(&snake_case_to_camel_case(&field_name.to_string()), Span::call_site());
                    let field_flatten = is_flatten(&field.attrs);
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

    let filter_ident = concatenate_idents(&name, &Ident::new("Filter", Span::call_site()));
    let all_filter_ident = concatenate_idents(&name, &Ident::new("AllFilter", Span::call_site()));
    let any_filter_ident = concatenate_idents(&name, &Ident::new("AnyFilter", Span::call_site()));
    let fields_filter_ident =
        concatenate_idents(&name, &Ident::new("FieldsFilter", Span::call_site()));

    let filter_fields = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().filter_map(|field| {
                let field_name = field.ident.clone().unwrap();
                let field_ty = &field.ty;
                let field_flatten = is_flatten(&field.attrs);
                if is_skip_filter(&field.attrs) {
                    return None;
                }
                if !field_flatten {
                    if is_skip_graphql(&field.attrs) {
                        Some(quote! {
                            #[graphql(skip)]
                            pub #field_name: ::std::option::Option<<#field_ty as ::laser::filter::Filterable>::Filter>
                        })
                    } else {
                        Some(quote! {
                            pub #field_name: ::std::option::Option<<#field_ty as ::laser::filter::Filterable>::Filter>
                        })
                    }
                } else {
                    if is_skip_graphql(&field.attrs) {
                        Some(quote! {
                            #[graphql(flatten, skip)]
                            pub #field_name: <#field_ty as ::laser::filter::Filterable>::Filter
                        })
                    } else {
                        Some(quote! {
                            #[graphql(flatten)]
                            pub #field_name: <#field_ty as ::laser::filter::Filterable>::Filter
                        })
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
    let filter_fields_impl = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().filter_map(|field| {
                let field_name = field.ident.clone().unwrap();
                let field_flatten = is_flatten(&field.attrs);
                if is_skip_filter(&field.attrs) {
                    return None;
                }
                if !field_flatten {
                    Some(quote! {
                        if let Some(filter) = self.#field_name {
                            if needs_and {
                                qb.push(" AND ");
                            }
                            filter.into_sql(stringify!(#field_name), qb);
                            needs_and = true;
                        }
                    })
                } else {
                    Some(quote! {
                        let mut needs_and = self.#field_name.into_sql(needs_and, qb);
                    })
                }
            }),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };
    let filter_fields_builder_impl = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().filter_map(|field| {
                let field_name = field.ident.clone().unwrap();
                let with_field_name = concatenate_idents(&Ident::new("with_", Span::call_site()), &field_name);
                let field_ty = &field.ty;
                let field_flatten = is_flatten(&field.attrs);
                if is_skip_filter(&field.attrs) {
                    return None;
                }
                if !field_flatten {
                    Some(quote! {
                        pub fn #field_name(#field_name: <#field_ty as ::laser::filter::Filterable>::Filter) -> #fields_filter_ident {
                            #fields_filter_ident::#field_name(#field_name)
                        }
                    
                        pub fn #with_field_name(self, #field_name: <#field_ty as ::laser::filter::Filterable>::Filter) -> Self {
                            Self {
                                fields: #fields_filter_ident {
                                    #field_name: Some(#field_name.into()),
                                    ..self.fields
                                },
                                ..self
                            }
                        }
                    })
                } else {
                    Some(quote! {
                        pub fn #field_name(#field_name: <#field_ty as ::laser::filter::Filterable>::Filter) -> #fields_filter_ident {
                            #fields_filter_ident::#field_name(#field_name)
                        }
                    
                        pub fn #with_field_name(self, #field_name: <#field_ty as ::laser::filter::Filterable>::Filter) -> Self {
                            Self {
                                fields: #fields_filter_ident {
                                    #field_name: #field_name.into(),
                                    ..self.fields
                                },
                                ..self
                            }
                        }
                    })
                }
            }),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };
    let filter_fields_builder_impl_inner = match &ast.data {
        // Struct
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named.named.iter().filter_map(|field| {
                let field_name = field.ident.clone().unwrap();
                let with_field_name = concatenate_idents(&Ident::new("with_", Span::call_site()), &field_name);
                let field_ty = &field.ty;
                let field_flatten = is_flatten(&field.attrs);
                if is_skip_filter(&field.attrs) {
                    return None;
                }
                if !field_flatten {
                    Some(quote! {
                        pub fn #field_name(#field_name: <#field_ty as ::laser::filter::Filterable>::Filter) -> Self {
                            Self {
                                #field_name: Some(#field_name.into()),
                                ..Self::default()
                            }
                        }
                    
                        pub fn #with_field_name(self, #field_name: <#field_ty as ::laser::filter::Filterable>::Filter) -> Self {
                            Self {
                                #field_name: Some(#field_name.into()),
                                ..self
                            }
                        }
                    })
                } else {
                    Some(quote! {
                        pub fn #field_name(#field_name: <#field_ty as ::laser::filter::Filterable>::Filter) -> Self {
                            Self {
                                #field_name: #field_name.into(),
                                ..Self::default()
                            }
                        }
                    
                        pub fn #with_field_name(self, #field_name: <#field_ty as ::laser::filter::Filterable>::Filter) -> Self {
                            Self {
                                #field_name: #field_name.into(),
                                ..self
                            }
                        }
                    })
                }
            }),
            Fields::Unnamed(..) => {
                panic!("Laser cannot be derived for structs with unnamed fields")
            }
            Fields::Unit => panic!("Laser cannot be derived for unit-structs"),
        },
        _ => panic!("Laser cannot be derived for non-structs"),
    };

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let impl_table = if let Data::Struct(_) = &ast.data {
        if let Some(table_name) = is_table(&ast.attrs) {
            quote! {
                impl #impl_generics ::laser::table::Table for #name #ty_generics #where_clause {
                    fn table() -> ::laser::table::TableName {
                        ::laser::table::table(#table_name)
                    }
                }
            }
        } else {
            quote! {}
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #impl_table

        impl <'r> #impl_generics ::sqlx::FromRow<'r, ::sqlx::postgres::PgRow> for #name #ty_generics #where_clause {
            fn from_row(row: &'r ::sqlx::postgres::PgRow) -> ::sqlx::Result<Self> {
                use ::sqlx::Row;

                Ok(Self {
                    #(#try_froms,)*
                })
            }
        }

        impl #impl_generics ::laser::column::Columns for #name #ty_generics #where_clause {
            fn columns() -> impl ::std::iter::Iterator<Item = (::laser::column::ColumnName, bool)> {                
                #flattened_columns
                .chain([
                    #(#columns,)*
                ])
            }

            fn into_column_values(self, qb: &mut ::sqlx::QueryBuilder<'_, ::sqlx::Postgres>) {
                use ::laser::sql::IntoSql;

                #(#into_column_values;)*
            }
        }

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

        impl ::laser::filter::Filterable for #name {
            type Filter = #fields_filter_ident;
        }

        #[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::default::Default, ::async_graphql::InputObject)]
        #[graphql(rename_fields = "snake_case")]
        pub struct #filter_ident {
            #[graphql(flatten)]
            pub all: #all_filter_ident,
            #[graphql(flatten)]
            pub any: #any_filter_ident,
            #[graphql(flatten)]
            pub fields: #fields_filter_ident,
        }

        impl ::laser::sql::IntoSql for #filter_ident {
            fn into_sql(self, qb: &mut ::sqlx::QueryBuilder<'_, ::sqlx::Postgres>) {
                use ::laser::sql::IntoSql;

                self.fields
                    .clone()
                    .into_sql(self.any.clone().into_sql(self.all.clone().into_sql(false, qb), qb), qb);
            }
        }

        impl #filter_ident {
            pub fn all(all: impl ::std::convert::Into<::std::vec::Vec<#filter_ident>>) -> Self {
                Self {
                    all: #all_filter_ident { all: Some(all.into()) },
                    ..Self::default()
                }
            }

            pub fn any(any: impl ::std::convert::Into<::std::vec::Vec<#filter_ident>>) -> Self {
                Self {
                    any: #any_filter_ident { any: Some(any.into()) },
                    ..Self::default()
                }
            }

            #(#filter_fields_builder_impl)*
        }

        #[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::default::Default, ::async_graphql::InputObject)]
        #[graphql(rename_fields = "snake_case")]
        pub struct #all_filter_ident {
            pub all: ::std::option::Option<::std::vec::Vec<#filter_ident>>,
        }

        impl #all_filter_ident  {
            pub fn into_sql(self, needs_and: bool, qb: &mut ::sqlx::QueryBuilder<'_, ::sqlx::Postgres>) -> bool {
                use ::laser::sql::IntoSql;

                if let Some(all) = self.all {
                    if all.len() > 0 {
                        if needs_and {
                            qb.push(" AND ");
                        }
                        qb.push("(");
                        for (i, filter) in all.into_iter().enumerate() {
                            if i > 0 {
                                qb.push(" AND ");
                            }
                            qb.push("(");
                            filter.into_sql(qb);
                            qb.push(")");
                        }
                        qb.push(")");
                        return true;
                    }
                }
                return false;
            }
        }

        #[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::default::Default, ::async_graphql::InputObject)]
        #[graphql(rename_fields = "snake_case")]
        pub struct #any_filter_ident {
            pub any: ::std::option::Option<::std::vec::Vec<#filter_ident>>,
        }

        impl #any_filter_ident {
            pub fn into_sql(self, needs_and: bool, qb: &mut ::sqlx::QueryBuilder<'_, ::sqlx::Postgres>) -> bool {
                use ::laser::sql::IntoSql;

                if let Some(any) = self.any {
                    if any.len() > 0 {
                        if needs_and {
                            qb.push(" AND ");
                        }
                        qb.push("(");
                        for (i, filter) in any.into_iter().enumerate() {
                            if i > 0 {
                                qb.push(" OR ");
                            }
                            qb.push("(");
                            filter.into_sql(qb);
                            qb.push(")");
                        }
                        qb.push(")");
                        return true;
                    }
                }
                return false;
            }
        }

        #[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::default::Default, ::async_graphql::InputObject)]
        #[graphql(rename_fields = "snake_case")]
        pub struct #fields_filter_ident {
            #(#filter_fields,)*
        }

        impl ::std::convert::From<#fields_filter_ident> for #filter_ident {
            fn from(fields: #fields_filter_ident) -> Self {
                Self {
                    fields,
                    all: #all_filter_ident::default(),
                    any: #any_filter_ident::default(),
                }
            }
        }

        impl ::laser::sql::IntoSql for #fields_filter_ident {
            fn into_sql(self, qb: &mut ::sqlx::QueryBuilder<'_, ::sqlx::Postgres>) {
                self.into_sql(false, qb);
            }
        }

        impl #fields_filter_ident {
            pub fn into_sql(self, mut needs_and: bool, qb: &mut ::sqlx::QueryBuilder<'_, ::sqlx::Postgres>) -> bool {
                use ::laser::sql::IntoSql;
                #(#filter_fields_impl)*
                return needs_and;
            }

            #(#filter_fields_builder_impl_inner)*
        }
    };

    TokenStream::from(expanded)
}


#[proc_macro_derive(Filterable, attributes(laser))]
pub fn derive_filter(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    let name_with_filter = concatenate_idents(&name, &Ident::new("Filter", Span::call_site()));
    let name_attrs = cmp_attrs(&ast.attrs);

    let variants = name_attrs.iter().map(|attr| {
        match attr.as_str() {
            "=" => quote! { Eq(#name) },
            "<>" => quote! { Ne(#name) },
            "<" => quote! { Lt(#name) },
            "<=" => quote! { Le(#name) },
            ">" => quote! { Gt(#name) },
            ">=" => quote! { Ge(#name) },
            _ => panic!("invalid filter attribute, must be one of '=', '<>', '<', '<=', '>', or '>='"),
        }
    });

    let match_arms = name_attrs.iter().map(|attr| {
        match attr.as_str() {
            "=" => quote! { 
                #name_with_filter::Eq(x) => {
                    qb.push(column_name);
                    qb.push(" = ");
                    qb.push_bind(x);
                }
            },
            "<>" => quote! { 
                #name_with_filter::Ne(x) => {
                    qb.push(column_name);
                    qb.push(" <> ");
                    qb.push_bind(x);
                }
            },
            "<" => quote! { 
                #name_with_filter::Lt(x) => {
                    qb.push(column_name);
                    qb.push(" < ");
                    qb.push_bind(x);
                }
            },
            "<=" => quote! { 
                #name_with_filter::Le(x) => {
                    qb.push(column_name);
                    qb.push(" <= ");
                    qb.push_bind(x);
                }
            },
            ">" => quote! { 
                #name_with_filter::Gt(x) => {
                    qb.push(column_name);
                    qb.push(" > ");
                    qb.push_bind(x);
                }
            },
            ">=" => quote! { 
                #name_with_filter::Ge(x) => {
                    qb.push(column_name);
                    qb.push(" >= ");
                    qb.push_bind(x);
                }
            },
            _ => panic!("invalid filter attribute, must be one of '=', '<>', '<', '<=', '>', or '>='"),
        }
    }).collect::<Vec<_>>();

    let expanded = quote! {
        impl ::laser::filter::Filterable for #name {
            type Filter = #name_with_filter;
        }

        #[derive(::std::clone::Clone, ::std::fmt::Debug, ::async_graphql::OneofObject)]
        #[graphql(rename_fields = "snake_case")]
        pub enum #name_with_filter {
            #(#variants,)*
        }

        impl #name_with_filter {
            pub fn into_sql(self, column_name: &'static str, qb: &mut ::sqlx::QueryBuilder<'_, ::sqlx::Postgres>) {
                use ::laser::sql::IntoSql;

                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn is_pk(attrs: &Vec<Attribute>) -> bool {
    for attr in attrs {
        if attr.path().is_ident("laser") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    if meta_list
                        .tokens
                        .clone()
                        .into_iter()
                        .find(|t| "primary_key" == t.to_string())
                        .is_some()
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    return false;
}

fn is_flatten(attrs: &Vec<Attribute>) -> bool {
    for attr in attrs {
        if attr.path().is_ident("laser") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    if meta_list
                        .tokens
                        .clone()
                        .into_iter()
                        .find(|t| "flatten" == t.to_string())
                        .is_some()
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    return false;
}

fn is_skip_filter(attrs: &Vec<Attribute>) -> bool {
    for attr in attrs {
        if attr.path().is_ident("laser") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    if meta_list
                        .tokens
                        .clone()
                        .into_iter()
                        .find(|t| "skip_filter" == t.to_string())
                        .is_some()
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    return false;
}

fn is_skip_sort_by(attrs: &Vec<Attribute>) -> bool {
    for attr in attrs {
        if attr.path().is_ident("laser") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    if meta_list
                        .tokens
                        .clone()
                        .into_iter()
                        .find(|t| "skip_sort_by" == t.to_string())
                        .is_some()
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    return false;
}

fn is_skip_graphql(attrs: &Vec<Attribute>) -> bool {
    for attr in attrs {
        if attr.path().is_ident("graphql") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    if meta_list
                        .tokens
                        .clone()
                        .into_iter()
                        .find(|t| "skip" == t.to_string())
                        .is_some()
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    return false;
}

fn is_table(attrs: &Vec<Attribute>) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("laser") {
            if let Meta::List(meta_list) = &attr.meta {
                let mut meta_list_token_iter = meta_list.tokens.clone().into_iter();
                match (
                    meta_list_token_iter
                        .next()
                        .map(|t| t.to_string())
                        .as_deref(),
                    meta_list_token_iter
                        .next()
                        .map(|t| t.to_string())
                        .as_deref(),
                    meta_list_token_iter.next().map(|t| t.to_string()),
                ) {
                    (Some("table"), Some("="), Some(table_name))
                        if table_name.starts_with('\"') && table_name.ends_with('\"') =>
                    {
                        return Some(table_name[1..table_name.len() - 1].to_owned());
                    }
                    _ => {}
                }
            }
        }
    }
    None
}


fn cmp_attrs(attrs: &Vec<Attribute>) -> Vec<String> {
    for attr in attrs {
        if attr.path().is_ident("laser") {
            if let Meta::List(meta_list) =  &attr.meta {
                return meta_list.tokens.clone().into_iter().filter_map(|t| {
                    let cmp = t.to_string();
                    if cmp.starts_with('\"') && cmp.ends_with('\"') {
                        match cmp.as_str() {
                            "\"=\"" => Some("=".to_string()),
                            "\"<>\"" => Some("<>".to_string()),
                            "\"<\"" => Some("<".to_string()),
                            "\"<=\"" => Some("<=".to_string()),
                            "\">\"" => Some(">".to_string()),
                            "\">=\"" => Some(">=".to_string()),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }).collect();
            }
        }
    }
    vec![]
}

fn concatenate_idents(ident1: &Ident, ident2: &Ident) -> Ident {
    let combined = format!("{}{}", ident1, ident2);
    Ident::new(&combined, Span::call_site())
}

fn snake_case_to_camel_case(s: &str) -> String {
    s.split('_')
     .map(|word| {
         let mut c = word.chars();
         c.next()
          .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
          .unwrap_or_default()
     })
     .collect()
}