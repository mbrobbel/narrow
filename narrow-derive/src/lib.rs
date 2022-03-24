use proc_macro::TokenStream;

/// Derive macro for the Array trait.
#[proc_macro_derive(Array, attributes(narrow))]
pub fn derive_array(_input: TokenStream) -> TokenStream {
    // let DeriveInput {
    //     vis,
    //     ident,
    //     generics,
    //     data,
    //     ..
    // } = parse_macro_input!(input as DeriveInput);

    todo!()
}
