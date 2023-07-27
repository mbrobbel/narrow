use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use proc_macro_crate::FoundCrate;
use syn::{parse_macro_input, DataEnum, DataStruct, DataUnion, DeriveInput};

mod r#enum;
mod r#struct;
mod union;
mod util;

const CRATE: &str = "narrow";

static NARROW: Lazy<String> = Lazy::new(|| match proc_macro_crate::crate_name(CRATE) {
    Ok(found) => match found {
        FoundCrate::Itself => "crate".to_string(),
        FoundCrate::Name(name) => name,
    },
    _ => CRATE.to_string(),
});

/// Derive macro for the ArrayType trait.
#[proc_macro_derive(ArrayType, attributes(narrow))]
pub fn derive_array_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        syn::Data::Struct(DataStruct { ref fields, .. }) => r#struct::derive(&input, fields),
        syn::Data::Enum(DataEnum { ref variants, .. }) => r#enum::derive(&input, variants),
        syn::Data::Union(DataUnion { ref fields, .. }) => union::derive(&input, fields),
    }
    .into()
}
