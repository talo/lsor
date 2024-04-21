use proc_macro::TokenStream;

pub(crate) mod filter;
pub(crate) mod row;
pub(crate) mod sort;
pub(crate) mod ty;
pub(crate) mod util;

#[proc_macro_derive(Filter, attributes(lsor))]
pub fn derive_filter(input: TokenStream) -> TokenStream {
    filter::expand_derive_filter(input)
}

#[proc_macro_derive(Row, attributes(lsor))]
pub fn derive_row(input: TokenStream) -> TokenStream {
    row::expand_derive_row(input)
}

#[proc_macro_derive(Sort)]
pub fn derive_sort(input: TokenStream) -> TokenStream {
    sort::expand_derive_sort(input)
}

#[proc_macro_derive(Type, attributes(sqlx))]
pub fn derive_type(input: TokenStream) -> TokenStream {
    ty::expand_derive_type(input)
}
