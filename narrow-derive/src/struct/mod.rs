use proc_macro2::TokenStream;
use syn::{DeriveInput, Fields};

mod unit;

pub(super) fn derive(input: &DeriveInput, fields: &Fields) -> TokenStream {
    match fields {
        Fields::Unit => unit::derive(input),
        _ => todo!("non unit structs derive"),
    }
}
