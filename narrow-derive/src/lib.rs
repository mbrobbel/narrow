use proc_macro::TokenStream;
use proc_macro_crate::FoundCrate;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DataEnum, DataStruct, DataUnion, DeriveInput, Fields, FieldsNamed,
    FieldsUnnamed, Ident, Type, Variant,
};

// todo(mb): https://docs.rs/frunk/0.3.2/frunk/labelled/trait.Transmogrifier.html with array wrappers
// todo(mb): support generics
// todo(mb): trait bounds in where clause when generic is type argument of other type e.g. Option<T>
// https://github.com/serde-rs/serde/blob/master/serde_derive/src/bound.rs
// todo(mb): convert iterators into original data structures e.g. Vec<String> from list array iterator (requires GATs)
// todo(mb): support unnamed fields in iterator
// todo(mb): enum support (unit variant, struct variants, multiple fields in one variant)
// todo(mb): allow Option<enum type>
// - by wrapping the variant with i8::default type_id in a nullable (if it wasn't already)
// - or use an available nullable field and write that instead of i8::default for nulls
// todo(mb): derive arrayindex for struct arrays

/// Derive macro for the Array trait.
#[proc_macro_derive(Array, attributes(narrow))]
pub fn derive_array(input: TokenStream) -> TokenStream {
    let DeriveInput {
        vis, ident, data, ..
    } = parse_macro_input!(input as DeriveInput);

    let narrow =
        match proc_macro_crate::crate_name("narrow").expect("narrow is present in `Cargo.toml`") {
            FoundCrate::Itself => quote!(::narrow),
            FoundCrate::Name(name) => {
                let name = format_ident!("{}", name);
                quote!(::#name)
            }
        };

    match &data {
        Data::Struct(DataStruct { fields, .. }) => {
            match fields {
                Fields::Unit => {
                    todo!("unit struct")
                }
                Fields::Named(FieldsNamed { named: fields, .. })
                | Fields::Unnamed(FieldsUnnamed {
                    unnamed: fields, ..
                }) => {
                    // Get the field identifiers and types of the fields.
                    let (fields, ty): (Vec<_>, Vec<_>) = fields
                        .iter()
                        .enumerate()
                        .map(|(idx, field)| {
                            (
                                field
                                    .ident
                                    .clone()
                                    .unwrap_or_else(|| format_ident!("_{}", idx)),
                                field.ty.clone(),
                            )
                        })
                        .unzip();

                    assert!(
                        !fields.is_empty(),
                        "todo struct without fields are not yet supported"
                    );
                    let first_field = fields.first().unwrap();

                    // Create the raw array struct.
                    let wrapper_ident = format_ident!("Raw{}Array", &ident);
                    let alias_ident = format_ident!("{}Array", &ident);
                    let iter_ident = format_ident!("{}ArrayIter", &ident);

                    let tokens = quote! {
                        #[automatically_derived]
                        #[derive(Debug)]
                        #vis struct #wrapper_ident {
                            #(
                                pub #fields: <#ty as #narrow::ArrayType>::Array,
                            )*
                        }

                        #[automatically_derived]
                        #vis type #alias_ident<const N: bool> = #narrow::StructArray<#ident, N>;

                        #[automatically_derived]
                        impl #narrow::StructArrayType for #ident {
                            type Array = #wrapper_ident;
                        }

                        #[automatically_derived]
                        impl #narrow::ArrayType for #ident {
                            type Array = #narrow::StructArray<#ident, false>;
                        }

                        #[automatically_derived]
                        impl #narrow::ArrayData for #wrapper_ident {
                            fn len(&self) -> usize {
                                #narrow::ArrayData::len(&self.#first_field)
                            }
                            fn is_null(&self, index: usize) -> bool {
                                // Validity is tracked in StructArray's Validity.
                                false
                            }
                            fn null_count(&self) -> usize {
                                // See `is_null`.
                                0
                            }
                            fn is_valid(&self, index: usize) -> bool {
                                // See `is_null`.
                                true
                            }
                            fn valid_count(&self) -> usize {
                                // See `len` and `is_null`.
                                #narrow::ArrayData::len(&self.#first_field)
                            }
                        }

                        #[automatically_derived]
                        impl FromIterator<#ident> for #wrapper_ident
                        {
                            fn from_iter<I>(iter: I) -> Self
                            where
                                I: ::std::iter::IntoIterator<Item = #ident>,
                            {
                                let iter = iter.into_iter();
                                let (lower_bound, upper_bound) = iter.size_hint();
                                let capacity = upper_bound.unwrap_or(lower_bound);
                                #(
                                    let mut #fields = ::std::vec::Vec::with_capacity(capacity);
                                )*
                                for item in iter.into_iter() {
                                    #(
                                        #fields.push(item.#fields);
                                    )*
                                };
                                Self {
                                    #(
                                        #fields: #fields.into_iter().collect(),
                                    )*
                                }
                            }
                        }

                        #[automatically_derived]
                        #vis struct #iter_ident<'array> {
                            #(
                                #fields: <&'array <#ty as #narrow::ArrayType>::Array as ::std::iter::IntoIterator>::IntoIter,
                            )*
                        }

                        #[automatically_derived]
                        impl<'array> ::std::iter::Iterator for #iter_ident<'array> {
                            type Item = #ident;

                            fn next(&mut self) -> Option<Self::Item> {
                                Some(#ident {
                                    #(
                                        #fields: self.#fields.next()?.into(),
                                    )*
                                })
                            }

                            fn size_hint(&self) -> (usize, Option<usize>) {
                                self.#first_field.size_hint()
                            }
                        }

                        #[automatically_derived]
                        impl<'array> ::std::iter::IntoIterator for &'array #wrapper_ident {
                            type Item = #ident;
                            type IntoIter = #iter_ident<'array>;

                            fn into_iter(self) -> Self::IntoIter {
                                #iter_ident {
                                    #(
                                        #fields: self.#fields.into_iter(),
                                    )*
                                }
                            }
                        }
                    };

                    tokens.into()
                }
            }
        }
        Data::Enum(DataEnum { variants, .. }) => {
            struct EnumVariant<'a> {
                idx: usize,
                idx_ident: Ident,
                ident: &'a Ident,
                field: Type,
            }
            let variants = variants
                .iter()
                .enumerate()
                .map(|(idx, Variant { ident, fields, .. })| EnumVariant {
                    idx,
                    idx_ident: format_ident!("_{}", idx),
                    ident,
                    field: match fields {
                        Fields::Unit => todo!("unit variant"),
                        Fields::Named(FieldsNamed { named: _fields, .. }) => {
                            todo!("struct variant")
                        }
                        Fields::Unnamed(FieldsUnnamed {
                            unnamed: fields, ..
                        }) if fields.len() == 1 => fields.first().unwrap().ty.clone(),
                        Fields::Unnamed(_) => {
                            todo!("tuple variants with more than one field")
                        }
                    },
                })
                .collect::<Vec<_>>();

            let idx = variants
                .iter()
                .map(|EnumVariant { idx, .. }| *idx as i8)
                .collect::<Vec<_>>();
            let idx_ident = variants
                .iter()
                .map(|EnumVariant { idx_ident, .. }| idx_ident)
                .collect::<Vec<_>>();
            let idents = variants
                .iter()
                .map(|EnumVariant { ident, .. }| ident)
                .collect::<Vec<_>>();
            let ty = variants
                .iter()
                .map(|EnumVariant { field, .. }| field)
                .collect::<Vec<_>>();
            let num_variants = idents.len();

            let wrapper_ident = format_ident!("{}ArrayWrapper", &ident);

            (quote! {
                #[automatically_derived]
                impl From<&#ident> for i8 {
                    fn from(ident: &#ident) -> Self {
                        match ident {
                            #(
                                #ident::#idents(..) => #idx,
                            )*
                        }
                    }
                }

                #[automatically_derived]
                impl #narrow::UnionArrayType<true> for #ident {
                    type Child = #wrapper_ident<true>;
                    type Array = #narrow::DenseUnionArray<#ident, #num_variants>;
                }

                #[automatically_derived]
                impl #narrow::UnionArrayType<false> for #ident {
                    type Child = #wrapper_ident<false>;
                    type Array = #narrow::SparseUnionArray<#ident>;
                }

                #[automatically_derived]
                #[allow(non_snake_case)]
                #[derive(Debug)]
                #vis struct #wrapper_ident<const D: bool> {
                    #(
                        #idents: <#ty as #narrow::ArrayType>::Array,
                    )*
                }

                #[automatically_derived]
                impl<const D: bool> #narrow::Array for #wrapper_ident<D> {
                    type Validity = Self;

                    fn validity(&self) -> &Self::Validity {
                        self
                    }
                }

                #[automatically_derived]
                impl<const D: bool> #narrow::UnionArrayIndex<#ident> for #wrapper_ident<D> {
                        fn index(&self, type_id: i8, index: i32) -> #ident {
                            match type_id {
                                #(
                                    #idx => #ident::#idents(#narrow::ArrayIndex::index(&self.#idents, index as usize)),
                                )*
                                _ => unreachable!(),
                            }
                        }
                }

                #[automatically_derived]
                impl FromIterator<#ident> for #wrapper_ident<false> {
                    fn from_iter<I>(iter: I) -> Self
                    where
                        I: ::std::iter::IntoIterator<Item = #ident>,
                    {
                        let iter = iter.into_iter();
                        let (lower_bound, upper_bound) = iter.size_hint();
                        let capacity = upper_bound.unwrap_or(lower_bound);

                        #(
                            let mut #idx_ident = Vec::with_capacity(capacity);
                        )*

                        for item in iter {
                            #(
                                if let #ident::#idents(ref x) = item {
                                    #idx_ident.push(x.clone());
                                } else {
                                    #idx_ident.push(Default::default());
                                }
                            )*
                        }

                        Self {
                            #(
                                #idents: #idx_ident.into_iter().collect(),
                            )*
                        }

                    }
                }

                #[automatically_derived]
                impl FromIterator<#ident> for #wrapper_ident<true> {
                    fn from_iter<I>(iter: I) -> Self
                    where
                        I: ::std::iter::IntoIterator<Item = #ident>,
                    {
                        let iter = iter.into_iter();

                        #(
                            let mut #idx_ident = Vec::new();
                        )*

                        for item in iter {
                            match item {
                                #(
                                    #ident::#idents(x) => #idx_ident.push(x),
                                )*
                            }
                        }

                        Self {
                            #(
                                #idents: #idx_ident.into_iter().collect(),
                            )*
                        }

                    }
                }

            })
            .into()
        }
        Data::Union(DataUnion { fields: _, .. }) => {
            todo!("untagged unions")
        }
    }
}
