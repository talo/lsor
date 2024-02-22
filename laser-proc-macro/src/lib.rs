use proc_macro::TokenStream;

mod row;

#[proc_macro_derive(Row)]
pub fn derive_row(input: TokenStream) -> TokenStream {
    row::derive_row(input)
}
