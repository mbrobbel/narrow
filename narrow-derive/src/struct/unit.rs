use crate::{
    r#struct::{array_type_ident, array_type_impl},
    util,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_quote, DeriveInput, Generics, Ident, ImplGenerics, TypeGenerics, Visibility, WhereClause,
};

pub(super) fn derive(input: &DeriveInput) -> TokenStream {
    let DeriveInput {
        vis,
        ident,
        generics,
        ..
    } = input;

    // Generate the ArrayType implementation.
    let array_type_impl = array_type_impl(ident, generics);

    // Generate the StructArrayType implementation.
    let struct_array_type_impl = struct_array_type_impl(ident, generics);

    // Generate the Unit implementation.
    let unit_impl = unit_impl(ident, generics);

    // Generate the array type definition.
    let array_type_def = array_type_def(vis, ident, generics);

    // Generate the length impl.
    let length_impl = length_impl(ident, generics);

    // Generate the FromIterator impl.
    let from_iterator_impl = from_iterator_impl(ident, generics);

    // Generate the Extend impl.
    let extend_impl = extend_impl(ident, generics);

    // Generate the Default impl.
    let default_impl = default_impl(ident, generics);

    quote! {
      #array_type_impl

      #struct_array_type_impl

      #unit_impl

      #array_type_def

      #from_iterator_impl

      #length_impl

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

fn struct_array_type_impl(ident: &Ident, generics: &Generics) -> TokenStream {
    let narrow = util::narrow();

    let array_type_ident = array_type_ident(ident);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let array_generics = NullArrayGenerics::new(generics);
    let (_, ty_generics_with_buffer, _) = array_generics.split_for_impl();

    quote! {
        impl #impl_generics #narrow::array::StructArrayType for #ident #ty_generics #where_clause {
            type Array<Buffer: #narrow::buffer::BufferType> = #array_type_ident #ty_generics_with_buffer;
        }
    }
}

fn array_type_def(vis: &Visibility, ident: &Ident, generics: &Generics) -> TokenStream {
    let narrow = util::narrow();

    let array_type_ident = array_type_ident(ident);

    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let array_generics = NullArrayGenerics::new(generics);
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

    let array_generics = NullArrayGenerics::new(generics);
    let (impl_generics_with_buffer, ty_generics_with_buffer, _) = array_generics.split_for_impl();

    quote! {
        impl #impl_generics_with_buffer ::std::iter::FromIterator<#ident #ty_generics> for #array_type_ident #ty_generics_with_buffer #where_clause {
            fn from_iter<_I: ::std::iter::IntoIterator<Item = #ident #ty_generics>>(iter: _I) -> Self {
                Self(iter.into_iter().collect())
            }
        }
    }
}

fn length_impl(ident: &Ident, generics: &Generics) -> TokenStream {
    let narrow = util::narrow();

    let array_type_ident = array_type_ident(ident);

    let (_, _ty_generics, where_clause) = generics.split_for_impl();

    let array_generics = NullArrayGenerics::new(generics);
    let (impl_generics_with_buffer, ty_generics_with_buffer, _) = array_generics.split_for_impl();

    quote! {
        impl #impl_generics_with_buffer #narrow::Length for #array_type_ident #ty_generics_with_buffer #where_clause {
            #[inline]
            fn len(&self) -> usize {
                self.0.len()
            }
        }
    }
}

fn extend_impl(ident: &Ident, generics: &Generics) -> TokenStream {
    let array_type_ident = array_type_ident(ident);

    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let array_generics = NullArrayGenerics::new(generics);
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

    let array_generics = NullArrayGenerics::new(generics);
    let (impl_generics_with_buffer, ty_generics_with_buffer, _) = array_generics.split_for_impl();

    quote! {
        impl #impl_generics_with_buffer ::std::default::Default for #array_type_ident #ty_generics_with_buffer #where_clause {
            fn default() -> Self {
                Self(::std::default::Default::default())
            }
        }
    }
}

struct NullArrayGenerics(Generics);

impl NullArrayGenerics {
    fn new(generics: &Generics) -> Self {
        let narrow = util::narrow();

        let mut generics_with_buffer = generics.clone();
        generics_with_buffer
            .params
            .push(parse_quote!(Buffer: #narrow::buffer::BufferType = #narrow::buffer::VecBuffer));
        Self(generics_with_buffer)
    }
    fn split_for_impl(&self) -> (ImplGenerics, TypeGenerics, Option<&WhereClause>) {
        self.0.split_for_impl()
    }
}
