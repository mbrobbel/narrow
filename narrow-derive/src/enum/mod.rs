use proc_macro2::TokenStream;
use syn::{punctuated::Punctuated, token, DeriveInput, Variant};

pub(super) fn derive(
    _input: &DeriveInput,
    _variants: &Punctuated<Variant, token::Comma>,
) -> TokenStream {
    todo!("enum derive")
}
