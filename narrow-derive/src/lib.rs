use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_crate::FoundCrate;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, token::Comma, ConstParam, Data,
    DataEnum, DataStruct, DataUnion, DeriveInput, Field, Fields, FieldsUnnamed, GenericParam,
    Generics, Ident, Index, TypeParam, Variant, Visibility, WhereClause,
};

// todo(mb): trait bounds in where clause when generic is type argument of other type e.g. Option<T>
// https://github.com/serde-rs/serde/blob/master/serde_derive/src/bound.rs
// todo(mb): convert iterators into original data structures e.g. Vec<String> from list array iterator (requires GATs)
// todo(mb): support unnamed fields in iterator
// todo(mb): enum support (unit variant, struct variants, multiple fields in one variant)
// todo(mb): allow Option<enum type>
// - by wrapping the variant with i8::default type_id in a nullable (if it wasn't already)
// - or use an available nullable field and write that instead of i8::default for nulls
// todo(mb): derive arrayindex for struct arrays

fn narrow_crate() -> TokenStream2 {
    // todo(mb): cache
    match proc_macro_crate::crate_name("narrow").expect("narrow crate not found") {
        FoundCrate::Itself => quote!(::narrow),
        FoundCrate::Name(name) => {
            let name = format_ident!("{}", name);
            quote!(::#name)
        }
    }
}

fn derive_struct(
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    fields: &Fields,
) -> TokenStream2 {
    let narrow = narrow_crate();
    let array_ident = format_ident!("Raw{}Array", &ident);
    let alias_ident = format_ident!("{}Array", &ident);
    match fields {
        // Unit structs are mapped to NullArrays.
        Fields::Unit => {
            quote! {
                #[automatically_derived]
                #[derive(Debug)]
                #vis struct #array_ident(#narrow::NullArray<#ident>);

                #[automatically_derived]
                #vis type #alias_ident<const N: bool> = #narrow::StructArray<#ident, N>;

                #[automatically_derived]
                impl #narrow::StructArrayType for #ident {
                    type Array = #array_ident;
                }

                #[automatically_derived]
                impl #narrow::ArrayType for #ident {
                    type Array = #narrow::StructArray<#ident, false>;
                }

                #[automatically_derived]
                impl #narrow::ArrayData for #array_ident {
                    fn len(&self) -> usize {
                        self.0.len()
                    }
                    fn is_null(&self, index: usize) -> bool {
                        false
                    }
                    fn null_count(&self) -> usize {
                        0
                    }
                    fn is_valid(&self, index: usize) -> bool {
                        true
                    }
                    fn valid_count(&self) -> usize {
                        self.len()
                    }
                }

                #[automatically_derived]
                impl FromIterator<#ident> for #array_ident
                {
                    fn from_iter<I>(iter: I) -> Self
                    where
                        I: ::std::iter::IntoIterator<Item = #ident>,
                    {
                        Self(iter.into_iter().collect())
                    }
                }

                #[automatically_derived]
                impl<'array> ::std::iter::IntoIterator for &'array #array_ident {
                    type Item = #ident;
                    type IntoIter = ::std::iter::Map<::std::ops::Range<usize>, fn(usize) -> #ident>;

                    fn into_iter(self) -> Self::IntoIter {
                        self.0.into_iter()
                    }
                }
            }
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            // todo(mb): check if the num of fields is not 0
            let mut base_generics = generics.clone();
            let (_, base_type_generics, _) = base_generics.split_for_impl();
            let mut generics = generics;
            generics
                .type_params_mut()
                .for_each(|TypeParam { bounds, .. }| {
                    bounds.push(parse_quote!(#narrow::ArrayType));
                });
            // Add bounds for all derived traits on the array struct.
            // todo(mb): maybe force this on the array trait definition level
            let mut generics_with_bound = generics.clone();
            generics.make_where_clause();
            generics.params.iter().for_each(|param| {
                if let GenericParam::Type(TypeParam { ident, .. }) = param {
                    generics.where_clause.as_mut().iter_mut().for_each(
                        |WhereClause { predicates, .. }| {
                            // todo(mb): not sure if I want the from_iter requirement here or only on from_iter_impl where clause
                            predicates.push(parse_quote!(
                                <#ident as #narrow::ArrayType>::Array: FromIterator<#ident> + Debug
                            ));
                        },
                    );
                }
            });
            let (impl_generics, type_generics, _) = generics.split_for_impl();
            let where_clause = &generics.where_clause;
            let field_vis = unnamed.into_iter().map(|Field { vis, .. }| vis);
            let field_ty = unnamed.into_iter().map(|Field { ty, .. }| ty);
            let field_idx = (0..fields.len()).map(Index::from).collect::<Vec<_>>();
            let field_idx_ident = field_idx
                .iter()
                .map(|x| format_ident!("_{}", x))
                .collect::<Vec<_>>();
            let array_ident_def = quote!(
                #[automatically_derived]
                #[derive(Debug)]
                #vis struct #array_ident #generics(
                    #(
                        #field_vis <#field_ty as #narrow::ArrayType>::Array,
                    )*
                ) #where_clause;

            );
            // Add const N bool generic arg to array_alias generics
            let mut type_alias_generics = generics.clone();
            let nullability_const_ident =
                if generics.params.iter().any(
                    |generic_param| matches!(generic_param, GenericParam::Const(ConstParam { ident, .. }) if ident == "N"),
                ) {
                    format_ident!("NARROW_N") // todo(mb): attribute
                } else {
                    format_ident!("N")
                };
            type_alias_generics.params.push(GenericParam::Const(
                parse_quote!(const #nullability_const_ident: bool),
            ));
            // todo(mb): https://github.com/rust-lang/rust/issues/21903
            let type_alias_def = quote!(
                #[automatically_derived]
                #vis type #alias_ident #type_alias_generics = #narrow::StructArray<#ident #base_type_generics, #nullability_const_ident>;
            );

            let impl_struct_array_type = quote!(
                #[automatically_derived]
                impl #impl_generics #narrow::StructArrayType for #ident #base_type_generics #where_clause {
                    type Array = #array_ident #type_generics;
                }
            );

            let impl_array_type = quote!(
                #[automatically_derived]
                impl #impl_generics #narrow::ArrayType for #ident #base_type_generics #where_clause {
                    type Array = #narrow::StructArray<#ident #base_type_generics, false>;
                }
            );

            let impl_array_data = quote!(
                #[automatically_derived]
                impl #impl_generics #narrow::ArrayData for #array_ident #type_generics #where_clause {
                    fn len(&self) -> usize {
                        #narrow::ArrayData::len(&self.0)
                    }
                    fn is_null(&self, index: usize) -> bool {
                        false
                    }
                    fn null_count(&self) -> usize {
                        0
                    }
                    fn is_valid(&self, index: usize) -> bool {
                        true
                    }
                    fn valid_count(&self) -> usize {
                        #narrow::ArrayData::len(&self.0)
                    }
                }
            );

            let from_iter_impl = quote!(
                #[automatically_derived]
                impl #impl_generics FromIterator<#ident #base_type_generics> for #array_ident #type_generics #where_clause
                {
                    fn from_iter<I>(iter: I) -> Self
                    where
                        I: ::std::iter::IntoIterator<Item = #ident #base_type_generics>,
                    {
                        let iter = iter.into_iter();
                        let (lower_bound, upper_bound) = iter.size_hint();
                        let capacity = upper_bound.unwrap_or(lower_bound);
                        #(
                            let mut #field_idx_ident = ::std::vec::Vec::with_capacity(capacity);
                        )*
                        for item in iter.into_iter() {
                            #(
                                #field_idx_ident.push(item.#field_idx);
                            )*
                        };
                        Self(
                            #(
                                #field_idx_ident.into_iter().collect(),
                            )*
                        )
                    }
                }
            );

            let iter_ident = format_ident!("{}ArrayIter", &ident);
            let field_type = fields.iter().map(|Field { ty, .. }| ty);
            let mut iter_generics = generics.clone();
            iter_generics.params.push(parse_quote!('array));
            let (impl_iter_generics, type_iter_generics, _) = iter_generics.split_for_impl();
            generics_with_bound.make_where_clause();
            generics_with_bound.params.iter().for_each(|param| {
                if let GenericParam::Type(TypeParam { ident, .. }) = param {
                    generics_with_bound
                        .where_clause
                        .as_mut()
                        .iter_mut()
                        .for_each(|WhereClause { predicates, .. }| {
                            predicates.push(parse_quote!(
                                &'array #ident::Array: ::std::iter::IntoIterator
                            ));
                        });
                }
            });
            let into_iter_where_clause = generics_with_bound.where_clause;

            let iter_def = quote!(
                #[automatically_derived]
                #vis struct #iter_ident #impl_iter_generics(
                    #(
                        <&'array <#field_type as #narrow::ArrayType>::Array as ::std::iter::IntoIterator>::IntoIter,
                    )*
                ) #into_iter_where_clause;
            );

            // let iter_iter_impl = quote!(
            //     #[automatically_derived]
            //     impl #impl_iter_generics ::std::iter::Iterator for #iter_ident #type_iter_generics #into_iter_where_clause {
            //         type Item = #ident #type_generics;

            //         fn next(&mut self) -> Option<Self::Item> {
            //             Some(#ident(
            //                 #(
            //                     self.#field_idx.next()?.into(),
            //                 )*
            //             ))
            //         }

            //         fn size_hint(&self) -> (usize, Option<usize>) {
            //             self.0.size_hint()
            //         }
            //     }
            // );

            // let into_iter_impl = quote!(
            //     #[automatically_derived]
            //     impl #impl_iter_generics ::std::iter::IntoIterator for &'array #array_ident #type_generics #into_iter_where_clause {
            //         type Item = #ident #type_generics;
            //         type IntoIter = #iter_ident #type_iter_generics;

            //         fn into_iter(self) -> Self::IntoIter {
            //             #iter_ident(
            //                 #(
            //                     self.#field_idx.into_iter(),
            //                 )*
            //             )
            //         }
            //     }
            // );

            let out = quote!(
                #array_ident_def
                #type_alias_def
                #impl_struct_array_type
                #impl_array_type
                #impl_array_data
                #from_iter_impl
                // #iter_def
                // #iter_iter_impl
                // #into_iter_impl
            );
            println!("{}", &out);
            out
        }
        _ => todo!(),
    }
}

fn derive_enum(
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    variants: &Punctuated<Variant, Comma>,
) -> TokenStream2 {
    let narrow = narrow_crate();
    let array_ident = format_ident!("Raw{}Array", &ident);
    let alias_ident = format_ident!("{}Array", &ident);

    let num_variants = variants.len();
    let num_variants_i8 = i8::try_from(num_variants).expect("More than 127 variants");

    let num_fields = variants
        .iter()
        .map(|Variant { fields, .. }| fields.len())
        .sum::<usize>();
    let has_fields = num_fields != 0;

    let variant_idents = variants
        .iter()
        .map(|Variant { ident, .. }| ident)
        .collect::<Vec<_>>();
    let variant_idx = 0..num_variants_i8;

    match num_variants {
        0 => unimplemented!("zero-variant enums"),
        // one-variant enums
        1 => {
            if !has_fields {
                // can't have generics
                quote!(
                    #[automatically_derived]
                    #[allow(non_snake_case)]
                    #[derive(Debug)]
                    #vis struct #array_ident<const D: bool> {
                        #(
                            #variant_idents: #narrow::NullArray<#ident>,
                        )*
                    }

                    #[automatically_derived]
                    #vis type #alias_ident<const D: bool> = #array_ident<D>;

                    #[automatically_derived]
                    impl #narrow::UnionArrayVariants for #ident {
                        const VARIANTS: usize = #num_variants;
                    }

                    #[automatically_derived]
                    impl #narrow::UnionArrayType<true> for #ident {
                        type Child = #array_ident<true>;
                        type Array = #narrow::DenseUnionArray<#ident, #num_variants>;
                    }

                    #[automatically_derived]
                    impl #narrow::UnionArrayType<false> for #ident {
                        type Child = #array_ident<false>;
                        type Array = #narrow::SparseUnionArray<#ident>;
                    }

                    #[automatically_derived]
                    impl From<&#ident> for i8 {
                        fn from(ident: &#ident) -> Self {
                            match ident {
                                #(
                                    #ident::#variant_idents => #variant_idx,
                                )*
                            }
                        }
                    }

                    #[automatically_derived]
                    impl<const D: bool> #narrow::Array for #array_ident<D> {
                        type Validity = Self;

                        fn validity(&self) -> &Self::Validity {
                            self
                        }
                    }

                    #[automatically_derived]
                    impl<const D: bool> #narrow::UnionArrayIndex<#ident> for #array_ident<D> {
                        fn index(&self, type_id: i8, index: i32) -> #ident {
                            #[cold]
                            #[inline(never)]
                            fn assert_failed(index: usize, len: usize) -> ! {
                                panic!("index (is {}) should be < len (is {})", index, len);
                            }

                            let len = self.#(#variant_idents)*.len();
                            if index as usize >= len {
                                assert_failed(index as usize, len);
                            }

                            #(
                                #ident::#variant_idents
                            )*
                        }
                    }

                    #[automatically_derived]
                    impl FromIterator<#ident> for #array_ident<false> {
                        fn from_iter<I>(iter: I) -> Self
                        where
                            I: ::std::iter::IntoIterator<Item = #ident>,
                        {
                            Self {
                                #(
                                    #variant_idents: iter.into_iter().collect()
                                )*
                            }
                        }
                    }

                    #[automatically_derived]
                    impl FromIterator<#ident> for #array_ident<true> {
                        fn from_iter<I>(iter: I) -> Self
                        where
                            I: ::std::iter::IntoIterator<Item = #ident>,
                        {
                            Self {
                                #(
                                    #variant_idents: iter.into_iter().collect()
                                )*
                            }
                        }
                    }
                )
            } else {
                quote!()
            }
        }
        // multi-variant enums
        _ => {
            quote!()
        }
    }
}

/// Derive macro for the Array trait.
#[proc_macro_derive(Array, attributes(narrow))]
pub fn derive_array(input: TokenStream) -> TokenStream {
    let DeriveInput {
        vis,
        ident,
        generics,
        data,
        ..
    } = parse_macro_input!(input as DeriveInput);

    match &data {
        Data::Struct(DataStruct { fields, .. }) => derive_struct(vis, ident, generics, fields),
        Data::Enum(DataEnum { variants, .. }) => derive_enum(vis, ident, generics, variants),
        Data::Union(DataUnion { .. }) => unimplemented!("untagged unions"),
    }
    .into()
}

//     {
//         match fields {
//             Fields::Unit => {
//                 todo!("unit struct")
//             }
//             Fields::Named(FieldsNamed { named: fields, .. })
//             | Fields::Unnamed(FieldsUnnamed {
//                 unnamed: fields, ..
//             }) => {
//                 // Get the field identifiers and types of the fields.
//                 let (fields, ty): (Vec<_>, Vec<_>) = fields
//                     .iter()
//                     .enumerate()
//                     .map(|(idx, field)| {
//                         (
//                             field
//                                 .ident
//                                 .clone()
//                                 .unwrap_or_else(|| format_ident!("_{}", idx)),
//                             field.ty.clone(),
//                         )
//                     })
//                     .unzip();

//                 assert!(
//                     !fields.is_empty(),
//                     "todo struct without fields are not yet supported"
//                 );
//                 let first_field = fields.first().unwrap();

//                 // Create the raw array struct.
//                 let wrapper_ident = format_ident!("Raw{}Array", &ident);
//                 let alias_ident = format_ident!("{}Array", &ident);
//                 let iter_ident = format_ident!("{}ArrayIter", &ident);

//                 let tokens = quote! {
//                     #[automatically_derived]
//                     #[derive(Debug)]
//                     #vis struct #wrapper_ident {
//                         #(
//                             pub #fields: <#ty as #narrow::ArrayType>::Array,
//                         )*
//                     }

//                     #[automatically_derived]
//                     #vis type #alias_ident<const N: bool> = #narrow::StructArray<#ident, N>;

//                     #[automatically_derived]
//                     impl #narrow::StructArrayType for #ident {
//                         type Array = #wrapper_ident;
//                     }

//                     #[automatically_derived]
//                     impl #narrow::ArrayType for #ident {
//                         type Array = #narrow::StructArray<#ident, false>;
//                     }

//                     #[automatically_derived]
//                     impl #narrow::ArrayData for #wrapper_ident {
//                         fn len(&self) -> usize {
//                             #narrow::ArrayData::len(&self.#first_field)
//                         }
//                         fn is_null(&self, index: usize) -> bool {
//                             // Validity is tracked in StructArray's Validity.
//                             false
//                         }
//                         fn null_count(&self) -> usize {
//                             // See `is_null`.
//                             0
//                         }
//                         fn is_valid(&self, index: usize) -> bool {
//                             // See `is_null`.
//                             true
//                         }
//                         fn valid_count(&self) -> usize {
//                             // See `len` and `is_null`.
//                             #narrow::ArrayData::len(&self.#first_field)
//                         }
//                     }

//                     #[automatically_derived]
//                     impl FromIterator<#ident> for #wrapper_ident
//                     {
//                         fn from_iter<I>(iter: I) -> Self
//                         where
//                             I: ::std::iter::IntoIterator<Item = #ident>,
//                         {
//                             let iter = iter.into_iter();
//                             let (lower_bound, upper_bound) = iter.size_hint();
//                             let capacity = upper_bound.unwrap_or(lower_bound);
//                             #(
//                                 let mut #fields = ::std::vec::Vec::with_capacity(capacity);
//                             )*
//                             for item in iter.into_iter() {
//                                 #(
//                                     #fields.push(item.#fields);
//                                 )*
//                             };
//                             Self {
//                                 #(
//                                     #fields: #fields.into_iter().collect(),
//                                 )*
//                             }
//                         }
//                     }

//                     #[automatically_derived]
//                     #vis struct #iter_ident<'array> {
//                         #(
//                             #fields: <&'array <#ty as #narrow::ArrayType>::Array as ::std::iter::IntoIterator>::IntoIter,
//                         )*
//                     }

//                     #[automatically_derived]
//                     impl<'array> ::std::iter::Iterator for #iter_ident<'array> {
//                         type Item = #ident;

//                         fn next(&mut self) -> Option<Self::Item> {
//                             Some(#ident {
//                                 #(
//                                     #fields: self.#fields.next()?.into(),
//                                 )*
//                             })
//                         }

//                         fn size_hint(&self) -> (usize, Option<usize>) {
//                             self.#first_field.size_hint()
//                         }
//                     }

//                     #[automatically_derived]
//                     impl<'array> ::std::iter::IntoIterator for &'array #wrapper_ident {
//                         type Item = #ident;
//                         type IntoIter = #iter_ident<'array>;

//                         fn into_iter(self) -> Self::IntoIter {
//                             #iter_ident {
//                                 #(
//                                     #fields: self.#fields.into_iter(),
//                                 )*
//                             }
//                         }
//                     }
//                 };

//                 tokens.into()
//             }
//         }
//     }
//     Data::Enum(DataEnum { variants, .. }) => {
//         struct EnumVariant<'a> {
//             idx: usize,
//             idx_ident: Ident,
//             ident: &'a Ident,
//             field: Type,
//         }
//         let variants = variants
//             .iter()
//             .enumerate()
//             .map(|(idx, Variant { ident, fields, .. })| EnumVariant {
//                 idx,
//                 idx_ident: format_ident!("_{}", idx),
//                 ident,
//                 field: match fields {
//                     Fields::Unit => todo!("unit variant"),
//                     Fields::Named(FieldsNamed { named: _fields, .. }) => {
//                         todo!("struct variant")
//                     }
//                     Fields::Unnamed(FieldsUnnamed {
//                         unnamed: fields, ..
//                     }) if fields.len() == 1 => fields.first().unwrap().ty.clone(),
//                     Fields::Unnamed(_) => {
//                         todo!("tuple variants with more than one field")
//                     }
//                 },
//             })
//             .collect::<Vec<_>>();

//         let idx = variants
//             .iter()
//             .map(|EnumVariant { idx, .. }| *idx as i8)
//             .collect::<Vec<_>>();
//         let idx_ident = variants
//             .iter()
//             .map(|EnumVariant { idx_ident, .. }| idx_ident)
//             .collect::<Vec<_>>();
//         let idents = variants
//             .iter()
//             .map(|EnumVariant { ident, .. }| ident)
//             .collect::<Vec<_>>();
//         let ty = variants
//             .iter()
//             .map(|EnumVariant { field, .. }| field)
//             .collect::<Vec<_>>();
//         let num_variants = idents.len();

//         let wrapper_ident = format_ident!("{}ArrayWrapper", &ident);

//         (quote! {
//             #[automatically_derived]
//             impl From<&#ident> for i8 {
//                 fn from(ident: &#ident) -> Self {
//                     match ident {
//                         #(
//                             #ident::#idents(..) => #idx,
//                         )*
//                     }
//                 }
//             }

//             #[automatically_derived]
//             impl #narrow::UnionArrayType<true> for #ident {
//                 type Child = #wrapper_ident<true>;
//                 type Array = #narrow::DenseUnionArray<#ident, #num_variants>;
//             }

//             #[automatically_derived]
//             impl #narrow::UnionArrayType<false> for #ident {
//                 type Child = #wrapper_ident<false>;
//                 type Array = #narrow::SparseUnionArray<#ident>;
//             }

//             #[automatically_derived]
//             #[allow(non_snake_case)]
//             #[derive(Debug)]
//             #vis struct #wrapper_ident<const D: bool> {
//                 #(
//                     #idents: <#ty as #narrow::ArrayType>::Array,
//                 )*
//             }

//             #[automatically_derived]
//             impl<const D: bool> #narrow::Array for #wrapper_ident<D> {
//                 type Validity = Self;

//                 fn validity(&self) -> &Self::Validity {
//                     self
//                 }
//             }

//             #[automatically_derived]
//             impl<const D: bool> #narrow::UnionArrayIndex<#ident> for #wrapper_ident<D> {
//                     fn index(&self, type_id: i8, index: i32) -> #ident {
//                         match type_id {
//                             #(
//                                 #idx => #ident::#idents(#narrow::ArrayIndex::index(&self.#idents, index as usize)),
//                             )*
//                             _ => unreachable!(),
//                         }
//                     }
//             }

//             #[automatically_derived]
//             impl FromIterator<#ident> for #wrapper_ident<false> {
//                 fn from_iter<I>(iter: I) -> Self
//                 where
//                     I: ::std::iter::IntoIterator<Item = #ident>,
//                 {
//                     let iter = iter.into_iter();
//                     let (lower_bound, upper_bound) = iter.size_hint();
//                     let capacity = upper_bound.unwrap_or(lower_bound);

//                     #(
//                         let mut #idx_ident = Vec::with_capacity(capacity);
//                     )*

//                     for item in iter {
//                         #(
//                             if let #ident::#idents(ref x) = item {
//                                 #idx_ident.push(x.clone());
//                             } else {
//                                 #idx_ident.push(Default::default());
//                             }
//                         )*
//                     }

//                     Self {
//                         #(
//                             #idents: #idx_ident.into_iter().collect(),
//                         )*
//                     }

//                 }
//             }

//             #[automatically_derived]
//             impl FromIterator<#ident> for #wrapper_ident<true> {
//                 fn from_iter<I>(iter: I) -> Self
//                 where
//                     I: ::std::iter::IntoIterator<Item = #ident>,
//                 {
//                     let iter = iter.into_iter();

//                     #(
//                         let mut #idx_ident = Vec::new();
//                     )*

//                     for item in iter {
//                         match item {
//                             #(
//                                 #ident::#idents(x) => #idx_ident.push(x),
//                             )*
//                         }
//                     }

//                     Self {
//                         #(
//                             #idents: #idx_ident.into_iter().collect(),
//                         )*
//                     }

//                 }
//             }

//         })
//         .into()
//     }
//     Data::Union(DataUnion { fields: _, .. }) => {
//         todo!("untagged unions")
//     }
// }
// }
