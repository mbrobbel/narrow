use crate::util;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::ops::Deref;
use syn::{
    parse_quote, punctuated::Punctuated, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed,
    Generics, Ident, Index, Token,
};

mod named;
mod unit;
mod unnamed;

pub(super) fn derive(input: &DeriveInput, fields: &Fields) -> TokenStream {
    let empty = Punctuated::new();
    let fields_inner = match fields {
        Fields::Unit => &empty,
        Fields::Unnamed(FieldsUnnamed {
            unnamed: fields, ..
        })
        | Fields::Named(FieldsNamed { named: fields, .. }) => fields,
    };
    let array_impl = match fields {
        Fields::Unit => unit::derive(input),
        Fields::Unnamed(FieldsUnnamed {
            unnamed: fields, ..
        }) => unnamed::derive(input, fields),
        Fields::Named(FieldsNamed { named: fields, .. }) => named::derive(input, fields),
    };

    let DeriveInput {
        ident, generics, ..
    } = input;

    // Generate the ArrayType implementation.
    let array_type_impl = array_type_impl(ident, generics);

    // Generate the StructArrayType implementation.
    let struct_array_type_impl = struct_array_type_impl(ident, generics, fields_inner);

    // Generate the Length implementation.
    let length_impl = length_impl(ident, generics, fields.iter().next());

    quote! {
        #array_type_impl

        #struct_array_type_impl

        #array_impl

        #length_impl
    }
}

fn array_type_ident(ident: &Ident) -> Ident {
    format_ident!("{}Array", ident)
}

fn array_type_impl(ident: &Ident, generics: &Generics) -> TokenStream {
    let narrow = util::narrow();

    let (_, ty_generics, _) = generics.split_for_impl();
    let array_type_bound_generics = ArrayTypeBound::from(generics);
    let (impl_generics, _, where_clause) = array_type_bound_generics.split_for_impl();

    quote! {
        impl #impl_generics #narrow::array::ArrayType for #ident #ty_generics #where_clause {
            type Array<Buffer: #narrow::buffer::BufferType> = #narrow::array::StructArray<#ident #ty_generics, false, Buffer>;
        }

        impl #impl_generics #narrow::array::ArrayType<#ident #ty_generics> for ::std::option::Option<#ident #ty_generics> #where_clause {
            type Array<Buffer: #narrow::buffer::BufferType> = #narrow::array::StructArray<#ident #ty_generics, true, Buffer>;
        }
    }
}

fn struct_array_type_impl(
    ident: &Ident,
    generics: &Generics,
    fields: &Punctuated<Field, Token![,]>,
) -> TokenStream {
    let narrow = util::narrow();
    let array_type_ident = array_type_ident(ident);

    let generics = ArrayTypeBound::from(generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let generics = BufferTypeGeneric::from(&*generics);
    let (_, ty_generics_with_buffer, _) = generics.split_for_impl();

    // Add bounds for all field types
    // let generics = ArrayTypeFieldWhereClause::from((&*generics, fields));
    // let (_, _, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #narrow::array::StructArrayType for #ident #ty_generics #where_clause {
            type Array<Buffer: #narrow::buffer::BufferType> = #array_type_ident #ty_generics_with_buffer;
        }
    }
}

fn length_impl(ident: &Ident, generics: &Generics, field: Option<&Field>) -> TokenStream {
    let narrow = util::narrow();

    let array_type_ident = array_type_ident(ident);

    let generics = ArrayTypeBound::from(generics);
    let (_, _ty_generics, _) = generics.split_for_impl();

    let array_generics = BufferTypeGeneric::from(&*generics);
    let (impl_generics_with_buffer, ty_generics_with_buffer, _) = array_generics.split_for_impl();

    // If there is a field we need to add a length bound in the where_clause
    let mut generics = generics.clone();
    let where_clause = match field {
        Some(Field { ty, .. }) => {
            let where_clause = generics.make_where_clause();
            // where_clause
            //     .predicates
            //     .push(parse_quote!(#ty: #narrow::array::ArrayType));
            where_clause.predicates.push(
                parse_quote!(<#ty as #narrow::array::ArrayType>::Array<Buffer>: #narrow::Length),
            );
            generics.where_clause
        }
        None => generics.where_clause,
    };

    // If there is named field we need to use the ident otherwise we use an index (0)
    let method = match field {
        Some(Field {
            ident: Some(ident), ..
        }) => {
            quote!(self.#ident.len())
        }
        None | Some(Field { ident: None, .. }) => {
            let ident = Index::from(0);
            quote!(self.#ident.len())
        }
    };

    quote! {
        impl #impl_generics_with_buffer #narrow::Length for #array_type_ident #ty_generics_with_buffer #where_clause {
            #[inline]
            fn len(&self) -> usize {
                #method
            }
        }
    }
}

// struct ArrayTypeFieldWhereClause(Generics);

// impl From<(&Generics, &Punctuated<Field, Token![,]>)> for ArrayTypeFieldWhereClause {
//     fn from((generics, fields): (&Generics, &Punctuated<Field, Token![,]>)) -> Self {
//         let narrow = util::narrow();
//         // Add bounds for all field types
//         let mut generics = generics.clone();
//         let where_clause = generics.make_where_clause();
//         where_clause
//             .predicates
//             .extend(fields.into_iter().map::<WherePredicate, _>(
//                 |Field { ty, .. }| parse_quote!(#ty: #narrow::array::ArrayType),
//             ));
//         Self(generics)
//     }
// }

// impl Deref for ArrayTypeFieldWhereClause {
//     type Target = Generics;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

/// Adds an `#narrow::array::ArrayType` bound to all type generics.
struct ArrayTypeBound(Generics);

impl Deref for ArrayTypeBound {
    type Target = Generics;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&Generics> for ArrayTypeBound {
    fn from(value: &Generics) -> Self {
        let narrow = util::narrow();
        let mut generics = value.clone();
        generics.type_params_mut().for_each(|type_param| {
            type_param
                .bounds
                .push(parse_quote!(#narrow::array::ArrayType));
        });
        Self(generics)
    }
}

/// Adds a `Buffer` generic with a `BufferType` bound.
struct BufferTypeGeneric(Generics);

impl Deref for BufferTypeGeneric {
    type Target = Generics;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&Generics> for BufferTypeGeneric {
    fn from(value: &Generics) -> Self {
        let narrow = util::narrow();
        let mut generics = value.clone();
        generics
            .params
            .push(parse_quote!(Buffer: #narrow::buffer::BufferType = #narrow::buffer::VecBuffer));
        Self(generics)
    }
}
