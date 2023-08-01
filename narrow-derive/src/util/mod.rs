use crate::NARROW;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

mod add_type_param;
pub(super) use add_type_param::*;

mod self_replace;
pub(super) use self_replace::*;

mod type_param_bound;
pub(super) use type_param_bound::*;

/// Returns the name of the `narrow` crate. Panics when the `narrow` crate is
/// not found.
pub(super) fn narrow() -> TokenStream {
    let ident = format_ident!("{}", &*NARROW);
    quote!(#ident)
}
