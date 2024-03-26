use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse2, parse_quote, punctuated::Punctuated, token, visit_mut::VisitMut, DeriveInput, Field,
    Fields, Generics, Ident, ItemImpl, ItemStruct, Token, Type, TypeParamBound, Variant,
    Visibility, WhereClause, WherePredicate,
};

use crate::util::{self, AddTypeParam, AddTypeParamBound, SelfReplace};

pub(super) fn derive(
    input: &DeriveInput,
    variants: &Punctuated<Variant, token::Comma>,
) -> TokenStream {
    let input = Enum::new(input, variants);

    // Generate the conversion to i8
    let i8_conversion = input.i8_conversion();

    // Generate the variant helper struct defs.
    let variant_struct_defs = input.variant_struct_defs();

    // Generate the enum variant impls.
    let enum_variant_impl = input.enum_variant_impl();

    // Generate the wrapper struct def.
    let array_struct_def = input.array_struct_def();

    // Generate a default impl for the wrapper struct def.
    let array_struct_default_impl = input.array_struct_default_impl();

    // Generate an extend impl for the wrapper struct def.
    let array_struct_extend_dense_impl = input.array_struct_extend_dense_impl();

    // Generate an extend impl for the wrapper struct def.
    let array_struct_extend_sparse_impl = input.array_struct_extend_sparse_impl();

    // Generate the UnionArrayType impl.
    let union_array_type_impl = input.union_array_type_impl();

    // Generate the ArrayType impl.
    let array_type_impl = input.array_type_impl();

    let tokens = quote! {
        #i8_conversion

        #variant_struct_defs

        #enum_variant_impl

        #array_struct_def

        #array_struct_default_impl

        #array_struct_extend_dense_impl

        #array_struct_extend_sparse_impl

        #union_array_type_impl

        #array_type_impl
    };

    #[cfg(feature = "arrow-rs")]
    {
        // // Generate the union array type fields impl.
        // let union_array_types_fields_impl = input.union_array_types_fields_impl();

        // // Generate the conversion to struct array.
        // let union_array_to_struct_array_impl = input.union_array_to_struct_array_impl();

        // // Generate the conversion from the variant arrays.
        // let union_array_from_struct_array_impl = input.union_array_from_struct_array_impl();

        quote! {
            #tokens

            // #union_array_types_fields_impl

            // #union_array_to_struct_array_impl

            // #union_array_from_struct_array_impl
        }
    }
    #[cfg(not(feature = "arrow-rs"))]
    tokens
}

struct Enum<'a> {
    vis: &'a Visibility,
    ident: &'a Ident,
    generics: &'a Generics,
    variants: &'a Punctuated<Variant, token::Comma>,
}

impl<'a> Enum<'a> {
    pub fn new(input: &'a DeriveInput, variants: &'a Punctuated<Variant, token::Comma>) -> Self {
        Self {
            vis: &input.vis,
            ident: &input.ident,
            generics: &input.generics,
            variants,
        }
    }
    fn variant_fields(&self) -> impl Iterator<Item = &Fields> + '_ {
        self.variants.iter().map(|variant| &variant.fields)
    }
    fn variant_idents(&self) -> impl Iterator<Item = &Ident> + '_ {
        self.variants.iter().map(|variant| &variant.ident)
    }
    fn variant_indices(&self) -> impl Iterator<Item = Literal> {
        (0..self.variants.len()).map(Literal::usize_unsuffixed)
    }
    fn variant_helper_idents_idents(&self) -> impl Iterator<Item = Ident> + '_ {
        self.variant_idents()
            .map(|ident| format_ident!("{}Variant{ident}", self.ident))
    }
    fn variant_helper_idents(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.variants.iter().map(|variant| {
            let ident = &variant.ident;
            match &variant.fields {
                Fields::Named(_) | Fields::Unnamed(_) => {
                    let ident = format_ident!("{}Variant{ident}", self.ident);
                    quote!(#ident)
                }
                Fields::Unit => quote!(()),
            }
        })
    }
    fn variant_pattern_ignore(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.variants.iter().map(|variant| {
            let ident = &variant.ident;
            match variant.fields {
                Fields::Named(_) => quote!(#ident{ .. }),
                Fields::Unnamed(_) => quote!(#ident(..)),
                Fields::Unit => quote!(#ident),
            }
        })
    }
    fn variant_field_iter(&self) -> impl Iterator<Item = impl Iterator<Item = Field>> + '_ {
        self.variant_fields().map(move |fields| match fields {
            Fields::Named(named) => named.named.clone().into_iter(),
            Fields::Unnamed(unnamed) => unnamed.unnamed.clone().into_iter(),
            Fields::Unit => Punctuated::<Field, Token![,]>::default().into_iter(),
        })
    }
    fn variant_helper_generics(&self) -> impl Iterator<Item = Generics> + '_ {
        self.variant_field_iter().map(|iter| {
            let mut generics = Generics::default();
            let ty_params = iter
                .filter_map(|field| match &field.ty {
                    Type::Path(path) => {
                        if let Some(ty) = path.path.get_ident() {
                            self.generics
                                .type_params()
                                .find(|ty_param| &ty_param.ident == ty)
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();

            ty_params.iter().for_each(|ty_param| {
                AddTypeParam(parse_quote!(#ty_param)).visit_generics_mut(&mut generics);
            });

            let mut self_generics = self.generics.clone();
            SelfReplace::new(self.ident, &self_generics).visit_generics_mut(&mut self_generics);
            let (_, _, where_clause) = self_generics.split_for_impl();
            generics.where_clause = where_clause
                .cloned()
                .map(|where_clause| {
                    where_clause
                        .predicates
                        .into_iter()
                        .filter(|predicate| {
                            if let WherePredicate::Type(ty) = predicate {
                                match &ty.bounded_ty {
                                    Type::Path(path) => {
                                        if let Some(ty) = path.path.get_ident() {
                                            ty_params.iter().any(|ty_param| &ty_param.ident == ty)
                                        } else {
                                            false
                                        }
                                    }
                                    _ => false,
                                }
                            } else {
                                false
                            }
                        })
                        .collect()
                })
                .map(|predicates| WhereClause {
                    where_token: token::Where::default(),
                    predicates,
                });

            generics
        })
    }

    /// Returns the name of the Array wrapper struct.
    fn array_struct_ident(&self) -> Ident {
        format_ident!("{}Array", self.ident)
    }

    /// Returns the `ArrayType` trait bound
    fn array_type_bound() -> TypeParamBound {
        let narrow = util::narrow();
        parse_quote!(#narrow::array::ArrayType)
    }

    /// Returns the implementation for the conversion into i8.
    fn i8_conversion(&self) -> ItemImpl {
        let ident = self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let variants = self
            .variant_pattern_ignore()
            .enumerate()
            .map(|(idx, pattern)| {
                let idx = Literal::usize_unsuffixed(idx);
                quote!(#pattern => #idx)
            });
        let tokens = quote! {
            impl #impl_generics ::std::convert::From<&#ident #ty_generics> for ::std::primitive::i8 #where_clause {
                fn from(value: &#ident #ty_generics) -> ::std::primitive::i8 {
                    match *value {
                        #(
                            #ident::#variants,
                        )*
                    }
                }
            }
        };
        parse2(tokens).expect("i8_conversion")
    }

    /// Returns the type definitions for the variant data helper structs.
    fn variant_struct_defs(&self) -> TokenStream {
        let narrow = util::narrow();

        let vis = self.vis;

        self.variant_fields()
            .zip(self.variant_helper_idents_idents())
            .zip(self.variant_helper_generics())
            .map(|((fields, ident), generics)| {
                let (impl_generics, _, where_clause) = generics.split_for_impl();
                match fields {
                    Fields::Named(named) => {
                        let field_ident = named.named.iter().map(|field| &field.ident);
                        let field_ty = named.named.iter().map(|field| &field.ty);
                        quote! {
                            #[derive(#narrow::ArrayType, Default)]
                            #vis struct #ident #impl_generics #where_clause {
                                #(
                                    #field_ident: #field_ty,
                                )*
                            }
                        }
                    }
                    Fields::Unnamed(unnamed) => {
                        let field_ty = unnamed.unnamed.iter().map(|field| &field.ty);
                        quote! {
                            #[derive(#narrow::ArrayType, Default)]
                            #vis struct #ident #impl_generics(
                                #(
                                    #field_ty,
                                )*
                            ) #where_clause;
                        }
                    }
                    Fields::Unit => quote! {},
                }
            })
            .collect()
    }

    /// Generates the enum variant impls.
    fn enum_variant_impl(&self) -> TokenStream {
        let narrow = util::narrow();
        let self_ident = self.ident;

        // Get the type generics to propagate to the corresponding structs.
        let (self_impl_generics, self_ty_generics, self_where_clause) =
            self.generics.split_for_impl();

        let idx = self.variant_indices();
        let variant_helper = self.variant_helper_idents();
        let variant_helper_generics = self.variant_helper_generics().collect::<Vec<_>>();
        let variant_ty_generics = variant_helper_generics.iter().map(|generics| {
            let (_, ty_generics, _) = generics.split_for_impl();
            ty_generics
        });
        let variant_ident = self.variant_idents();
        let fields = self.variants.iter().map(|variant| match &variant.fields {
            Fields::Named(named) => {
                let ident = named.named.iter().map(|field| &field.ident);
                quote! {
                    {
                        #(
                            #ident: value.#ident,
                        )*
                    }
                }
            }
            Fields::Unnamed(unnamed) => {
                let ty = unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| Literal::usize_unsuffixed(idx));
                quote! {
                    (
                        #(
                            value.#ty,
                        )*
                    )
                }
            }
            Fields::Unit => quote! {},
        });
        quote! {
            #(
                impl #self_impl_generics #narrow::array::union::EnumVariant<#idx> for #self_ident #self_ty_generics #self_where_clause {
                    type Data = #variant_helper #variant_ty_generics;
                    fn from_data(value: Self::Data) -> Self {
                        Self::#variant_ident #fields
                    }
                }
            )*
        }
    }

    /// Returns the struct definition of the Array wrapper struct.
    fn array_struct_def(&self) -> ItemStruct {
        let narrow = util::narrow();

        // Generics
        let self_generics = self.generics.clone();
        let (_, self_ty_generics, _) = self_generics.split_for_impl();
        let mut generics = self.generics.clone();
        SelfReplace::new(self.ident, &generics).visit_generics_mut(&mut generics);
        AddTypeParamBound(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(OffsetItem: #narrow::offset::OffsetElement))
            .visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(UnionLayout: #narrow::array::UnionType))
            .visit_generics_mut(&mut generics);
        let (impl_generics, _, where_clause) = generics.split_for_impl();

        let idx = self
            .variants
            .iter()
            .enumerate()
            .map(|(idx, _)| Literal::usize_unsuffixed(idx));

        let vis = self.vis;
        let self_ident = self.ident;
        let ident = self.array_struct_ident();
        let tokens = quote!(
            #vis struct #ident #impl_generics (
                #(
                  <<#self_ident #self_ty_generics as #narrow::array::union::EnumVariant<#idx>>::Data as #narrow::array::ArrayType<<#self_ident #self_ty_generics as #narrow::array::union::EnumVariant<#idx>>::Data>>::Array<Buffer, OffsetItem, UnionLayout>,
                )*
            ) #where_clause;
        );
        parse2(tokens).expect("array_struct_def")
    }

    // Adds a default impl for the array wrapper struct.
    fn array_struct_default_impl(&self) -> ItemImpl {
        let narrow = util::narrow();

        // Generics
        let self_generics = self.generics.clone();
        let (_, self_ty_generics, _) = self_generics.split_for_impl();
        let mut generics = self.generics.clone();
        SelfReplace::new(self.ident, &generics).visit_generics_mut(&mut generics);
        AddTypeParamBound(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(OffsetItem: #narrow::offset::OffsetElement))
            .visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(UnionLayout: #narrow::array::UnionType))
            .visit_generics_mut(&mut generics);
        let self_ident = self.ident;
        generics
            .make_where_clause()
            .predicates
            .extend(
                self.variant_indices()
                    .map::<WherePredicate, _>(|idx|
                        parse_quote!(
                            <<#self_ident #self_ty_generics as #narrow::array::union::EnumVariant<#idx>>::Data as #narrow::array::ArrayType<<#self_ident #self_ty_generics as #narrow::array::union::EnumVariant<#idx>>::Data>>::Array<Buffer, OffsetItem, UnionLayout>
                        : ::std::default::Default)
                    )
            );
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let ident = self.array_struct_ident();
        let default_fields = self
            .variants
            .iter()
            .map(|_| quote!(::std::default::Default::default()));
        let tokens = quote! {
            impl #impl_generics ::std::default::Default for #ident #ty_generics #where_clause {
                fn default() -> Self {
                    Self(
                        #(
                            #default_fields,
                        )*
                    )
                }
            }
        };
        parse2(tokens).expect("array_struct_default_impl")
    }

    // Adds an extend impl for the dense array wrapper struct.
    fn array_struct_extend_dense_impl(&self) -> ItemImpl {
        let narrow = util::narrow();

        // Generics
        let self_generics = self.generics.clone();
        let self_ident = self.ident;
        let (_, self_ty_generics, _) = self_generics.split_for_impl();
        let mut generics = self.generics.clone();
        SelfReplace::new(self.ident, &generics).visit_generics_mut(&mut generics);
        AddTypeParamBound(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(OffsetItem: #narrow::offset::OffsetElement))
            .visit_generics_mut(&mut generics);

        let struct_defs = self
            .variant_helper_idents()
            .zip(self.variant_helper_generics())
            .map(|(ident, generics)| {
                let (_, ty_generics, _) = generics.split_for_impl();
                quote! { #ident #ty_generics }
            });
        generics
            .make_where_clause()
            .predicates
            .extend(
                self.variant_indices().zip(struct_defs).map::<WherePredicate, _>(|(idx, struct_def)|{
                    parse_quote!(<<#self_ident #self_ty_generics as #narrow::array::union::EnumVariant<#idx>>::Data as #narrow::array::ArrayType<<#self_ident #self_ty_generics as #narrow::array::union::EnumVariant<#idx>>::Data>>::Array<Buffer, OffsetItem, #narrow::array::DenseLayout>: ::std::iter::Extend<#struct_def>)
                })
            );
        let (impl_generics, _, where_clause) = generics.split_for_impl();
        let mut generics = generics.clone();
        AddTypeParam(parse_quote!(DenseLayout)).visit_generics_mut(&mut generics);
        let (_, ty_generics, _) = generics.split_for_impl();
        let ident = self.array_struct_ident();
        let fields = self
            .variants
            .iter()
            .enumerate()
            .zip(self.variant_helper_idents_idents())
            .map(|((idx, variant), variant_ident)| {
                let idx = Literal::usize_unsuffixed(idx);
                let ident = &variant.ident;
                match &variant.fields {
                    Fields::Named(named) => {
                        let field_idents = named
                            .named
                            .iter()
                            .map(|field| &field.ident)
                            .collect::<Vec<_>>();
                        quote! {
                            #self_ident::#ident { #( #field_idents, )* } => {
                                self.#idx.extend(::std::iter::once(#variant_ident { #( #field_idents, )* }));
                            }
                        }
                    }
                    Fields::Unnamed(unnamed) => {
                        let field_idx = unnamed.unnamed.iter().enumerate().map(|(idx, _)| format_ident!("_{idx}")).collect::<Vec<_>>();
                        quote! {
                            #self_ident::#ident (#( #field_idx, )*) => {
                                self.#idx.extend(::std::iter::once(#variant_ident( #( #field_idx, )* )));
                            }
                        }
                    },
                    Fields::Unit => quote! {
                        #self_ident::#ident => {
                            self.#idx.extend(::std::iter::once(()));
                        }
                    },
                }
            });
        let tokens = quote! {
            impl #impl_generics ::std::iter::Extend<#self_ident #self_ty_generics> for #ident #ty_generics #where_clause {
                fn extend<I>(&mut self, iter: I) where I: IntoIterator<Item = #self_ident #self_ty_generics> {
                    iter.into_iter().for_each(|variant| {
                        match variant {
                            #(
                                #fields,
                            )*
                        }
                    });
                }
            }
        };
        parse2(tokens).expect("array_struct_extend_dense_impl")
    }

    // Adds an extend impl for the sparse array wrapper struct.
    fn array_struct_extend_sparse_impl(&self) -> ItemImpl {
        let narrow = util::narrow();

        // Generics
        let self_generics = self.generics.clone();
        let self_ident = self.ident;
        let (_, self_ty_generics, _) = self_generics.split_for_impl();
        let mut generics = self.generics.clone();
        SelfReplace::new(self.ident, &generics).visit_generics_mut(&mut generics);
        AddTypeParamBound(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(OffsetItem: #narrow::offset::OffsetElement))
            .visit_generics_mut(&mut generics);

        let struct_defs = self
            .variant_helper_idents()
            .zip(self.variant_helper_generics())
            .map(|(ident, generics)| {
                let (_, ty_generics, _) = generics.split_for_impl();
                quote! { #ident #ty_generics }
            });
        generics
            .make_where_clause()
            .predicates
            .extend(
                self.variant_indices().zip(struct_defs).map::<WherePredicate, _>(|(idx, struct_def)|{
                    parse_quote!(<<#self_ident #self_ty_generics as #narrow::array::union::EnumVariant<#idx>>::Data as #narrow::array::ArrayType<<#self_ident #self_ty_generics as #narrow::array::union::EnumVariant<#idx>>::Data>>::Array<Buffer, OffsetItem, #narrow::array::SparseLayout>: ::std::iter::Extend<#struct_def>)
                })
            );
        let (impl_generics, _, where_clause) = generics.split_for_impl();
        let mut generics = generics.clone();
        AddTypeParam(parse_quote!(SparseLayout)).visit_generics_mut(&mut generics);
        let (_, ty_generics, _) = generics.split_for_impl();
        let ident = self.array_struct_ident();
        let fields = self
            .variants
            .iter()
            .enumerate()
            .zip(self.variant_helper_idents_idents())
            .map(|((index, variant), variant_ident)| {
                let idx = Literal::usize_unsuffixed(index);
                let ident = &variant.ident;
                let other_idx = (0..self.variants.len()).filter(|&var_idx| index != var_idx).map(Literal::usize_unsuffixed);
                match &variant.fields {
                    Fields::Named(named) => {
                        let field_idents = named
                            .named
                            .iter()
                            .map(|field| &field.ident)
                            .collect::<Vec<_>>();
                        quote! {
                            #self_ident::#ident { #( #field_idents, )* } => {
                                self.#idx.extend(::std::iter::once(#variant_ident { #( #field_idents, )* }));
                                #(
                                    self.#other_idx.extend(::std::iter::once(::std::default::Default::default()));
                                )*
                            }
                        }
                    }
                    Fields::Unnamed(unnamed) => {
                        let field_idx = unnamed.unnamed.iter().enumerate().map(|(idx, _)| format_ident!("_{idx}")).collect::<Vec<_>>();
                        quote! {
                            #self_ident::#ident (#( #field_idx, )*) => {
                                self.#idx.extend(::std::iter::once(#variant_ident( #( #field_idx, )* )));
                                #(
                                    self.#other_idx.extend(::std::iter::once(::std::default::Default::default()));
                                )*
                            }
                        }
                    },
                    Fields::Unit => {
                        quote! {
                        #self_ident::#ident => {
                            self.#idx.extend(::std::iter::once(()));
                            #(
                                self.#other_idx.extend(::std::iter::once(::std::default::Default::default()));
                            )*
                        }
                    }},
                }
            });
        let tokens = quote! {
            impl #impl_generics ::std::iter::Extend<#self_ident #self_ty_generics> for #ident #ty_generics #where_clause {
                fn extend<I>(&mut self, iter: I) where I: IntoIterator<Item = #self_ident #self_ty_generics> {
                    iter.into_iter().for_each(|variant| {
                        match variant {
                            #(
                                #fields,
                            )*
                        }
                    });
                }
            }
        };
        parse2(tokens).expect("array_struct_extend_sparse_impl")
    }

    fn union_array_type_impl(&self) -> ItemImpl {
        let narrow = util::narrow();

        let variants = Literal::usize_unsuffixed(self.variants.len());

        // Generics
        let mut generics = self.generics.clone();
        AddTypeParamBound(Enum::array_type_bound()).visit_generics_mut(&mut generics);
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let mut array_generics = generics.clone();
        AddTypeParam(parse_quote!(Buffer)).visit_generics_mut(&mut array_generics);
        AddTypeParam(parse_quote!(OffsetItem)).visit_generics_mut(&mut array_generics);
        AddTypeParam(parse_quote!(UnionLayout)).visit_generics_mut(&mut array_generics);
        let (_, array_ty_generics, _) = array_generics.split_for_impl();

        let self_ident = self.ident;
        let ident = self.array_struct_ident();
        let tokens = quote! {
            impl #impl_generics #narrow::array::UnionArrayType<#variants> for #self_ident #ty_generics #where_clause {
                type Array<Buffer: #narrow::buffer::BufferType, OffsetItem: #narrow::offset::OffsetElement, UnionLayout: #narrow::array::UnionType> = #ident #array_ty_generics;
            }
        };
        parse2(tokens).expect("union_array_type_impl")
    }

    fn array_type_impl(&self) -> ItemImpl {
        let narrow = util::narrow();

        // Generics
        let mut generics = self.generics.clone();
        AddTypeParamBound(Enum::array_type_bound()).visit_generics_mut(&mut generics);
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let ident = self.ident;
        let variants = Literal::usize_unsuffixed(self.variants.len());
        let tokens = quote! {
            impl #impl_generics #narrow::array::ArrayType<#ident #ty_generics> for #ident #ty_generics #where_clause {
                type Array<Buffer: #narrow::buffer::BufferType, OffsetItem: #narrow::offset::OffsetElement, UnionLayout: #narrow::array::UnionType> = #narrow::array::UnionArray<Self, { <Self as #narrow::array::UnionArrayType<#variants>>::VARIANTS }, UnionLayout, Buffer>;
            }
        };
        parse2(tokens).expect("array_type_impl")
    }

    // #[cfg(feature = "arrow-rs")]
    // fn union_array_types_fields_impl(&self) -> ItemImpl {
    //     let narrow = util::narrow();

    //     let ident = self.array_struct_ident();
    //     let mut generics = self.generics.clone();
    //     AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
    //         .visit_generics_mut(&mut generics);
    //     AddTypeParam(parse_quote!(OffsetItem: #narrow::offset::OffsetElement))
    //         .visit_generics_mut(&mut generics);
    //     AddTypeParam(parse_quote!(UnionLayout: #narrow::array::UnionType))
    //         .visit_generics_mut(&mut generics);
    //     let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    //     let self_ident = self.ident;
    //     let idx = self.variant_indices().collect::<Vec<_>>();
    //     let variants = Literal::usize_unsuffixed(self.variants.len());
    //     let variant_idx = (0..self.variants.len()).map(|idx| idx.to_string());
    //     let tokens = quote! {
    //         impl #impl_generics #narrow::arrow::UnionArrayTypeFields<#variants> for #ident #ty_generics #where_clause {
    //             fn fields() -> ::arrow_schema::Fields {
    //                 ::arrow_schema::Fields::from(vec![
    //                     #(
    //                         <<<#self_ident as #narrow::array::union::EnumVariant<#idx>>::Data as #narrow::array::ArrayType>::Array<
    //                             Buffer,
    //                             OffsetItem,
    //                             SparseLayout,
    //                         > as #narrow::arrow::Array>::as_field(#variant_idx),
    //                     )*
    //                 ])
    //             }
    //             fn type_ids() -> [::std::primitive::i8; #variants] {
    //                 [
    //                     #(
    //                         #idx,
    //                     )*
    //                 ]
    //             }
    //         }
    //     };
    //     parse2(tokens).expect("union_array_types_fields_impl")
    // }

    // #[cfg(feature = "arrow-rs")]
    // fn union_array_to_struct_array_impl(&self) -> ItemImpl {
    //     let narrow = util::narrow();

    //     let ident = self.array_struct_ident();
    //     let mut generics = self.generics.clone();
    //     AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
    //         .visit_generics_mut(&mut generics);
    //     AddTypeParam(parse_quote!(OffsetItem: #narrow::offset::OffsetElement))
    //         .visit_generics_mut(&mut generics);

    //     let self_ident = self.ident;
    //     let (_, self_ty_generics, _) = self.generics.split_for_impl();
    //     generics.make_where_clause().predicates.extend(
    //         self.variant_indices().map::<WherePredicate, _>(|idx| {
    //             parse_quote!(::std::sync::Arc<dyn ::arrow_array::Array>: From<
    //             <<#self_ident #self_ty_generics as #narrow::array::union::EnumVariant<#idx>>::Data as #narrow::array::ArrayType>::Array<
    //                 Buffer,
    //                 OffsetItem,
    //                 SparseLayout,
    //             >,
    //         >)
    //         }),
    //     );

    //     let (impl_generics, _, where_clause) = generics.split_for_impl();
    //     let mut generics = generics.clone();
    //     AddTypeParam(parse_quote!(SparseLayout)).visit_generics_mut(&mut generics);
    //     let (_, ty_generics, _) = generics.split_for_impl();
    //     let idx = self.variant_indices();
    //     let variants = Literal::usize_unsuffixed(self.variants.len());
    //     let tokens = quote! {
    //         impl #impl_generics ::std::convert::From<#ident #ty_generics> for ::arrow_array::StructArray #where_clause {
    //             fn from(value: #ident #ty_generics) -> Self {
    //                 // Safety:
    //                 // - TODO
    //                 unsafe {
    //                     ::arrow_array::StructArray::new_unchecked(
    //                         <#ident #ty_generics as #narrow::arrow::UnionArrayTypeFields<#variants>>::fields(),
    //                         vec![
    //                             #(
    //                                 value.#idx.into(),
    //                             )*
    //                         ],
    //                         None,
    //                     )
    //                 }
    //             }
    //         }
    //     };
    //     parse2(tokens).expect("union_array_to_struct_array_impl")
    // }

    // #[cfg(feature = "arrow-rs")]
    // fn union_array_from_struct_array_impl(&self) -> ItemImpl {
    //     let narrow = util::narrow();
    //     let self_ident = self.ident;
    //     let (_, self_ty_generics, _) = self.generics.split_for_impl();
    //     let ident = self.array_struct_ident();
    //     let mut generics = self.generics.clone();
    //     AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
    //         .visit_generics_mut(&mut generics);
    //     AddTypeParam(parse_quote!(OffsetItem: #narrow::offset::OffsetElement))
    //         .visit_generics_mut(&mut generics);
    //     AddTypeParam(parse_quote!(UnionLayout: #narrow::array::UnionType))
    //         .visit_generics_mut(&mut generics);
    //     generics.make_where_clause().predicates.extend(
    //         self.variant_indices().map::<WherePredicate, _>(|idx| {
    //             parse_quote!(::std::sync::Arc<dyn ::arrow_array::Array>: Into<
    //             <<#self_ident #self_ty_generics as #narrow::array::union::EnumVariant<#idx>>::Data as #narrow::array::ArrayType>::Array<
    //                 Buffer,
    //                 OffsetItem,
    //                 SparseLayout,
    //             >,
    //         >)
    //         }),
    //     );
    //     let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    //     let idx = (0..self.variants.len()).map(|_| Literal::usize_suffixed(0));
    //     let tokens = quote! {
    //         impl #impl_generics ::std::convert::From<::std::sync::Arc<dyn arrow_array::Array>> for #ident #ty_generics #where_clause {
    //             fn from(value: ::std::sync::Arc<dyn arrow_array::Array>) -> Self {
    //                 let struct_array = ::arrow_array::StructArray::from(value.to_data());
    //                 let (_, mut arrays, _) = struct_array.into_parts();
    //                 Self(
    //                     #(
    //                         arrays.remove(#idx).into(),
    //                     )*
    //                 )
    //             }
    //         }
    //     };
    //     parse2(tokens).expect("union_array_from_struct_array_impl")
    // }
}
