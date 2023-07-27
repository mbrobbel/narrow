use crate::{
    r#struct::{array_type_ident, BufferTypeGeneric},
    util,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Generics, Ident, Visibility};

pub(super) fn derive(input: &DeriveInput) -> TokenStream {
    let DeriveInput {
        vis,
        ident,
        generics,
        ..
    } = input;

    // Generate the Unit implementation.
    let unit_impl = unit_impl(ident, generics);

    // Generate the array type definition.
    let array_type_def = array_type_def(vis, ident, generics);

    // Generate the FromIterator impl.
    let from_iterator_impl = from_iterator_impl(ident, generics);

    // Generate the Extend impl.
    let extend_impl = extend_impl(ident, generics);

    // Generate the Default impl.
    let default_impl = default_impl(ident, generics);

    quote! {
      #unit_impl

      #array_type_def

      #from_iterator_impl

      #extend_impl

      #default_impl
    }
}

fn unit_impl(ident: &Ident, generics: &Generics) -> TokenStream {
    let narrow = util::narrow();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        /// Safety:
        /// - This is a unit struct.
        unsafe impl #impl_generics #narrow::array::Unit for #ident #ty_generics #where_clause {}
    }
}

fn array_type_def(vis: &Visibility, ident: &Ident, generics: &Generics) -> TokenStream {
    let narrow = util::narrow();

    let array_type_ident = array_type_ident(ident);

    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let array_generics = BufferTypeGeneric::from(generics);
    let (impl_generics_with_buffer, _, _) = array_generics.split_for_impl();

    quote! {
        #vis struct #array_type_ident #impl_generics_with_buffer(
            #narrow::array::NullArray<#ident #ty_generics, false, Buffer>
        ) #where_clause;
    }
}

fn from_iterator_impl(ident: &Ident, generics: &Generics) -> TokenStream {
    let array_type_ident = array_type_ident(ident);

    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let array_generics = BufferTypeGeneric::from(generics);
    let (impl_generics_with_buffer, ty_generics_with_buffer, _) = array_generics.split_for_impl();

    quote! {
        impl #impl_generics_with_buffer ::std::iter::FromIterator<#ident #ty_generics> for #array_type_ident #ty_generics_with_buffer #where_clause {
            fn from_iter<_I: ::std::iter::IntoIterator<Item = #ident #ty_generics>>(iter: _I) -> Self {
                Self(iter.into_iter().collect())
            }
        }
    }
}

fn extend_impl(ident: &Ident, generics: &Generics) -> TokenStream {
    let array_type_ident = array_type_ident(ident);

    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let array_generics = BufferTypeGeneric::from(generics);
    let (impl_generics_with_buffer, ty_generics_with_buffer, _) = array_generics.split_for_impl();

    quote! {
        impl #impl_generics_with_buffer ::std::iter::Extend<#ident #ty_generics> for #array_type_ident #ty_generics_with_buffer #where_clause {
            fn extend<_I: ::std::iter::IntoIterator<Item=#ident #ty_generics>>(&mut self, iter: _I) {
                self.0.extend(iter)
            }
        }
    }
}

fn default_impl(ident: &Ident, generics: &Generics) -> TokenStream {
    let array_type_ident = array_type_ident(ident);

    let (_, _, where_clause) = generics.split_for_impl();

    let array_generics = BufferTypeGeneric::from(generics);
    let (impl_generics_with_buffer, ty_generics_with_buffer, _) = array_generics.split_for_impl();

    quote! {
        impl #impl_generics_with_buffer ::std::default::Default for #array_type_ident #ty_generics_with_buffer #where_clause {
            fn default() -> Self {
                Self(::std::default::Default::default())
            }
        }
    }
}
