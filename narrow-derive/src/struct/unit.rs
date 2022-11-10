use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, visit_mut::VisitMut, DeriveInput, Generics, Ident, Visibility};

use crate::util;

pub(super) fn derive(input: &DeriveInput) -> TokenStream {
    let DeriveInput {
        vis,
        ident,
        generics,
        ..
    } = input;

    // let narrow = util::narrow();

    // Construct the raw array wrapper.
    // let raw_array_def = quote! {
    //   #[doc = #raw_array_doc]
    //   #vis struct #raw_array_ident #array_generics
    // (#narrow::array::null::NullArray<#ident #ty_generics>) #array_where_clause;
    // };

    // Add NullArray generics.
    // let mut array_generics = generics.clone();
    // AddNullableConstGeneric.visit_generics_mut(&mut array_generics);

    // // let generics = SelfReplace::generics(&ident, &generics);
    // let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // // Replace `Self` with ident in where clauses.
    // let mut where_clause = where_clause.cloned();
    // where_clause
    //     .as_mut()
    //     .map(|where_clause|
    // SelfReplace(&ident).visit_where_clause_mut(where_clause));

    // let array_generics = SelfReplace::generics(ident, generics);
    // let (array_impl_generics, array_ty_generics, array_where_clause) =
    //     array_generics.split_for_impl();

    // // Add a type definition for the array of this type.
    // let alias_array_ident = util::alias_array_ident(&ident);
    // let alias_array = quote!(
    //   #[doc = #raw_array_doc]
    //   #[automatically_derived]
    //   #vis type #alias_array_ident #generics #where_clause =
    // #narrow::array::r#struct::StructArray<#ident #ty_generics>; );

    // Implement ArrayType for this type.
    // let array_type_impl = quote! {
    //   #[automatically_derived]
    //   impl #impl_generics #narrow::array::ArrayType for #ident #ty_generics
    // #where_clause {     type Array<const N: bool, T: OffsetValue> =
    // #raw_array_ident #array_generics;   }
    // };

    let raw_array = raw_array_def(vis, ident, generics);

    // todo impl unit for this type

    quote! {
      #raw_array

      // #array_type_impl
    }
    //   #alias_array
    // }
}

pub(super) fn raw_array_def(vis: &Visibility, ident: &Ident, generics: &Generics) -> TokenStream {
    let narrow = util::narrow();

    // Get the ident and doc for the raw array struct.
    let (raw_array_ident, raw_array_doc) = util::raw_array(ident);

    // Get the ty_generics of the inner type.
    let (_, ty_generics, where_clause) = generics.split_for_impl();

    // Add the const generic trait bound for nullability.
    let generics = NullArrayGenerics::generics(generics);
    let nullarray_nullable = NullArrayGenerics::nullable_generic();
    let nullarray_validity_bitmap_buffer = NullArrayGenerics::validity_bitmap_buffer_generic();
    // let (_, _, _) = generics.split_for_impl();

    quote! {
      #[doc = #raw_array_doc]
      #vis struct #raw_array_ident #generics(
        #narrow::array::null::NullArray<
          #ident #ty_generics,
          #nullarray_nullable,
          #nullarray_validity_bitmap_buffer
        >
      ) #where_clause;
    }
}

struct NullArrayGenerics;

impl NullArrayGenerics {
    fn generics(generics: &Generics) -> Generics {
        let mut generics = generics.clone();
        Self.visit_generics_mut(&mut generics);
        generics
    }

    fn nullable_generic() -> Ident {
        format_ident!("_NARROW_NULLABLE")
    }

    fn validity_bitmap_buffer_generic() -> Ident {
        format_ident!("_NARROW_VALIDITY_BITMAP_BUFFER")
    }
}

impl VisitMut for NullArrayGenerics {
    fn visit_generics_mut(&mut self, generics: &mut Generics) {
        let nullable = Self::nullable_generic();
        generics
            .params
            .push(parse_quote!(const #nullable: bool = false));

        let validity_bitmap_buffer = Self::validity_bitmap_buffer_generic();
        generics
            .params
            .push(parse_quote!(#validity_bitmap_buffer = Vec<u8>));
    }
}
