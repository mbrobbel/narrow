use proc_macro2::TokenStream;
use syn::{DeriveInput, FieldsNamed};

pub(crate) fn derive(_input: &DeriveInput, _fields: &FieldsNamed) -> TokenStream {
    todo!("union derive")
}
