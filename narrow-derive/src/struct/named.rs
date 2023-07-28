use crate::{
    r#struct::{array_type_ident, ArrayTypeBound, BufferTypeGeneric},
    util,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_quote, punctuated::Punctuated, DeriveInput, Field, Generics, Ident, Token, Visibility,
    WherePredicate,
};

pub(super) fn derive(input: &DeriveInput, fields: &Punctuated<Field, Token![,]>) -> TokenStream {
    let DeriveInput {
        vis,
        ident,
        generics,
        ..
    } = input;

    // Generate the array type definition.
    let array_type_def = array_type_def(vis, ident, generics, fields);

    // Generate the FromIterator impl.
    let from_iterator_impl = from_iterator_impl(ident, generics, fields);

    // Generate the Default impl.
    let default_impl = default_impl(ident, generics, fields);

    // Generate the Extend impl.
    let extend_impl = extend_impl(ident, generics, fields);

    quote! {
        #array_type_def

        #from_iterator_impl

        #default_impl

        #extend_impl
    }
}

fn array_type_def(
    vis: &Visibility,
    ident: &Ident,
    generics: &Generics,
    fields: &Punctuated<Field, Token![,]>,
) -> TokenStream {
    let narrow = util::narrow();

    let array_type_ident = array_type_ident(ident);

    // Add an ArrayType bound to all generics.
    let generics = ArrayTypeBound::from(generics);
    // Add a Buffer generic for the array type.
    let mut generics = BufferTypeGeneric::from(&*generics).0;
    // // Add where clause
    // let generics = ArrayTypeFieldWhereClause::from((&*generics, fields));

    generics.make_where_clause().predicates.extend(
        fields
            .into_iter()
            .map(|Field { ty, .. }| ty)
            .map::<WherePredicate, _>(|ty| parse_quote!(#ty: #narrow::array::ArrayType)),
    );

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let field_vis = fields.into_iter().map(|Field { vis, .. }| vis);
    let field_ident = fields.into_iter().map(|Field { ident, .. }| ident);
    let field_ty = fields.into_iter().map(|Field { ty, .. }| ty);

    quote! {
        #vis struct #array_type_ident #impl_generics #where_clause {
            #(
                #field_vis #field_ident: <#field_ty as #narrow::array::ArrayType>::Array<Buffer>,
            )*
        }
    }
}

fn from_iterator_impl(
    ident: &Ident,
    generics: &Generics,
    fields: &Punctuated<Field, Token![,]>,
) -> TokenStream {
    let narrow = util::narrow();

    let array_type_ident = array_type_ident(ident);

    let (_, ty_generics, _) = generics.split_for_impl();

    // Add an ArrayType bound to all generics.
    let generics = ArrayTypeBound::from(generics);

    // Add a Buffer generic for the array type.
    let generics = BufferTypeGeneric::from(&*generics);
    let (impl_generics, ty_generics_with_buffer, _) = generics.split_for_impl();

    // Add bounds for all field types
    // let mut generics = ArrayTypeFieldWhereClause::from((&*generics, fields)).0;

    let mut generics = generics.clone();
    let where_clause = generics.make_where_clause();
    where_clause.predicates.extend(
        fields.into_iter().map::<WherePredicate, _>(|Field { ty, .. }|
            parse_quote!(<#ty as #narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default + ::std::iter::Extend<#ty>)
        )
    );

    let field_ident = fields
        .into_iter()
        .map(|Field { ident, .. }| ident)
        .collect::<Vec<_>>();
    let mut tuple = field_ident.iter().map(ToTokens::to_token_stream);
    let initial = tuple.next_back().map(|last| quote!((#last,()))).unwrap();
    let tuple = tuple.rfold(initial, |acc, x| quote!((#x, #acc)));

    quote! {
        impl #impl_generics ::std::iter::FromIterator<#ident #ty_generics> for #array_type_ident #ty_generics_with_buffer #where_clause {
            fn from_iter<_I: ::std::iter::IntoIterator<Item = #ident #ty_generics>>(iter: _I) -> Self {
                let #tuple = iter.into_iter().map(|#ident { #(#field_ident,)* } | #tuple ).unzip();
                Self {
                    #(
                        #field_ident,
                    )*
                }
            }
        }
    }
}

fn default_impl(
    ident: &Ident,
    generics: &Generics,
    fields: &Punctuated<Field, Token![,]>,
) -> TokenStream {
    let narrow = util::narrow();

    let array_type_ident = array_type_ident(ident);

    // Add an ArrayType bound to all generics.
    let generics = ArrayTypeBound::from(generics);
    // Add a Buffer generic for the array type.
    let array_generics = BufferTypeGeneric::from(&*generics);
    let (impl_generics, ty_generics, _) = array_generics.split_for_impl();

    // Add bounds for all field types
    // let mut generics = ArrayTypeFieldWhereClause::from((&*generics, fields)).0;
    let mut generics = generics.clone();
    let where_clause = generics.make_where_clause();
    where_clause.predicates.extend(
        fields.into_iter().map::<WherePredicate, _>(|Field { ty, .. }|
            parse_quote!(<#ty as #narrow::array::ArrayType>::Array<Buffer>: ::std::default::Default)
        )
    );

    let field_ident = fields.into_iter().map(|Field { ident, .. }| ident);
    let default_field = fields
        .into_iter()
        .map(|_| quote!(::std::default::Default::default()));

    quote!(
        impl #impl_generics ::std::default::Default for #array_type_ident #ty_generics #where_clause {
            fn default() -> Self {
                Self{
                    #(
                        #field_ident: #default_field,
                    )*
                }
            }
        }
    )
}

fn extend_impl(
    ident: &Ident,
    generics: &Generics,
    fields: &Punctuated<Field, Token![,]>,
) -> TokenStream {
    let narrow = util::narrow();

    let array_type_ident = array_type_ident(ident);

    let (_, ty_generics, _) = generics.split_for_impl();

    // Add an ArrayType bound to all generics.
    let generics = ArrayTypeBound::from(generics);
    // Add a Buffer generic for the array type.
    let generics = BufferTypeGeneric::from(&*generics);

    let (impl_generics, ty_generics_with_buffer, _) = generics.split_for_impl();
    // Add bounds for all field types
    // let mut generics = ArrayTypeFieldWhereClause::from((&*generics, fields)).0;

    // Add bounds for all field types
    let mut generics = generics.clone();
    let where_clause = generics.make_where_clause();
    where_clause.predicates.extend(
        fields.into_iter().map::<WherePredicate, _>(|Field { ty, .. }|
            parse_quote!(<#ty as #narrow::array::ArrayType>::Array<Buffer>: ::std::iter::Extend<#ty>)
        )
    );

    let field_ident = fields
        .iter()
        .map(|Field { ident, .. }| ident)
        .collect::<Vec<_>>();

    quote! {
        impl #impl_generics ::std::iter::Extend<#ident #ty_generics> for #array_type_ident #ty_generics_with_buffer #where_clause {
            fn extend<_I: ::std::iter::IntoIterator<Item = #ident #ty_generics>>(&mut self, iter: _I) {
                iter.into_iter().for_each(|#ident { #( #field_ident, )* } | {
                    #(
                        self.#field_ident.extend(::std::iter::once(#field_ident));
                    )*
                });
            }
        }
    }
}
