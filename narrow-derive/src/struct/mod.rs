use crate::util;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields, Generics, Ident};

mod unit;

pub(super) fn derive(input: &DeriveInput, fields: &Fields) -> TokenStream {
    match fields {
        Fields::Unit => unit::derive(input),
        _ => todo!("non unit structs derive"),
    }
}

fn array_type_ident(ident: &Ident) -> Ident {
    format_ident!("{}Array", ident)
}

fn array_type_impl(ident: &Ident, generics: &Generics) -> TokenStream {
    let narrow = util::narrow();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #narrow::array::ArrayType for #ident #ty_generics #where_clause {
            type Array<Buffer: #narrow::buffer::BufferType> = #narrow::array::StructArray<#ident #ty_generics, false, Buffer>;
        }

        impl #impl_generics #narrow::array::ArrayType<#ident #ty_generics> for ::std::option::Option<#ident #ty_generics> #where_clause {
            type Array<Buffer: #narrow::buffer::BufferType> = #narrow::array::StructArray<#ident #ty_generics, true, Buffer>;
        }
    }
}
