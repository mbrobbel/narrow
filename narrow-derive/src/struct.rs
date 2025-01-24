use crate::util::{self, AddTypeParam, AddTypeParamBoundWithSelf, DropOuterParam, SelfReplace};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use std::iter::{Enumerate, Map};
use syn::{
    parse2, parse_quote, punctuated, token::Paren, visit_mut::VisitMut, DeriveInput, Field, Fields,
    Generics, Ident, Index, ItemImpl, ItemStruct, Type, TypeParamBound, Visibility, WherePredicate,
};

pub(super) fn derive(input: &DeriveInput, fields: &Fields) -> TokenStream {
    let input = Struct::new(input, fields);

    // If this is a unit struct we generate a unit impl.
    let unit_impl = matches!(fields, Fields::Unit)
        .then(|| input.unit_impl())
        .map(ToTokens::into_token_stream)
        .unwrap_or_default();

    // Generate the ArrayType impl.
    let array_type_impl = input.array_type_impl();

    // Generate the StructArrayType impl.
    let struct_array_type_impl = input.struct_array_type_impl();

    // Generate the array wrapper struct definition.
    let array_struct_def = input.array_struct_def();

    // Generate a `Clone` impl for the array wrapper struct.
    let array_clone_impl = input.array_clone_impl();

    // Generate a `Default` impl for the array wrapper struct.
    let array_default_impl = input.array_default_impl();

    // Generate the Length implementation.
    let array_len_impl = input.array_len_impl();

    // Generate the Extend implementation.
    let array_extend_impl = input.array_extend_impl();

    // Generate the FromIterator implementation.
    let array_from_iter_impl = input.array_from_iter_impl();

    // Generate the iter struct definition.
    let array_iter_struct_def = input.array_iter_struct_def();

    // Generate the iterator impl for the iter struct.
    let array_iter_iterator_impl = input.array_iter_iterator_impl();

    // Generate the IntoIterator implementation.
    let array_into_iter_impl = input.array_into_iter_impl();

    let tokens = quote! {
        #unit_impl

        #array_type_impl

        #struct_array_type_impl

        #array_struct_def

        #array_clone_impl

        #array_default_impl

        #array_len_impl

        #array_extend_impl

        #array_from_iter_impl

        #array_iter_struct_def

        #array_iter_iterator_impl

        #array_into_iter_impl
    };

    #[cfg(feature = "arrow-rs")]
    {
        // Optionally generate the StructArrayTypeFields impl.
        let struct_array_type_fields_impl = input.struct_array_type_fields_impl();

        // Optionally generates the conversion to vec of array refs
        let struct_array_into_array_refs = input.struct_array_into_array_refs();

        // Optionally generates the conversion from vec of array refs
        let struct_array_from_array_refs = input.struct_array_from_array_refs();

        quote! {
            #tokens

            #struct_array_type_fields_impl

            #struct_array_into_array_refs

            #struct_array_from_array_refs
        }
    }
    #[cfg(not(feature = "arrow-rs"))]
    tokens
}

type FieldIdents<'a> = Map<Enumerate<punctuated::Iter<'a, Field>>, fn((usize, &Field)) -> Ident>;

struct Struct<'a> {
    vis: &'a Visibility,
    ident: &'a Ident,
    generics: &'a Generics,
    fields: &'a Fields,
}

impl<'a> Struct<'a> {
    pub fn new(input: &'a DeriveInput, fields: &'a Fields) -> Self {
        Self {
            vis: &input.vis,
            ident: &input.ident,
            generics: &input.generics,
            fields,
        }
    }
}

impl Struct<'_> {
    /// Returns the name of the Array wrapper struct.
    fn array_struct_ident(&self) -> Ident {
        format_ident!("{}Array", self.ident)
    }

    /// Returns the name of the Array iter wrapper struct.
    fn array_iter_struct_ident(&self) -> Ident {
        format_ident!("{}ArrayIter", self.ident)
    }

    /// Returns the `ArrayType` trait bound
    fn array_type_bound() -> TypeParamBound {
        let narrow = util::narrow();
        parse_quote!(#narrow::array::ArrayType)
    }

    fn surround_with_delimiters(&self, input: TokenStream) -> TokenStream {
        let mut tokens = TokenStream::new();
        match self.fields {
            Fields::Named(named) => named
                .brace_token
                .surround(&mut tokens, |tokens| tokens.append_all(input)),
            Fields::Unnamed(unnamed) => unnamed
                .paren_token
                .surround(&mut tokens, |tokens| tokens.append_all(input)),
            Fields::Unit => {
                Paren::default().surround(&mut tokens, |tokens| tokens.append_all(input))
            }
        }
        tokens
    }

    fn field_types(&self) -> Map<punctuated::Iter<'_, Field>, fn(&Field) -> &Type> {
        self.fields.iter().map(|Field { ty, .. }| ty)
    }

    fn field_vis(&self) -> Map<punctuated::Iter<'_, Field>, fn(&Field) -> &Visibility> {
        self.fields.iter().map(|Field { vis, .. }| vis)
    }

    fn field_types_drop_option(&self) -> impl Iterator<Item = Type> + '_ {
        self.fields
            .iter()
            .clone()
            .map(|Field { ty, .. }| ty)
            .map(|ty| {
                let mut ty = ty.clone();
                DropOuterParam.visit_type_mut(&mut ty);
                ty.clone()
            })
    }

    fn field_idents(&self) -> FieldIdents {
        self.fields
            .iter()
            .enumerate()
            .map(|(idx, Field { ident, .. })| ident.clone().unwrap_or(format_ident!("_{idx}")))
    }

    /// Add a `Unit` impl for the derive input.
    fn unit_impl(&self) -> ItemImpl {
        let narrow = util::narrow();

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let ident = self.ident;
        let tokens = quote! {
            /// Safety:
            /// - This is a unit struct.
            unsafe impl #impl_generics #narrow::array::Unit for #ident #ty_generics #where_clause {
                type Item = Self;
            }
        };
        parse2(tokens).expect("unit_impl")
    }

    /// Add an `ArrayType` implementation for the derive input.
    fn array_type_impl(&self) -> TokenStream {
        let narrow = util::narrow();

        // Generics
        let mut generics = self.generics.clone();
        AddTypeParamBoundWithSelf(Struct::array_type_bound()).visit_generics_mut(&mut generics);
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let ident = self.ident;
        let non_nullable = quote! {
            impl #impl_generics #narrow::array::ArrayType<#ident #ty_generics> for #ident #ty_generics #where_clause {
                type Array<Buffer: #narrow::buffer::BufferType, OffsetItem: #narrow::offset::Offset, UnionLayout: #narrow::array::UnionType> = #narrow::array::StructArray<#ident #ty_generics, #narrow::NonNullable, Buffer>;
            }
        };
        let non_nullable: ItemImpl = parse2(non_nullable).expect("array_type_impl");

        let nullable = quote! {
            impl #impl_generics #narrow::array::ArrayType<#ident #ty_generics> for ::std::option::Option<#ident #ty_generics> #where_clause {
                type Array<Buffer: #narrow::buffer::BufferType, OffsetItem: #narrow::offset::Offset, UnionLayout: #narrow::array::UnionType> = #narrow::array::StructArray<#ident #ty_generics, #narrow::Nullable, Buffer>;
            }
        };
        let nullable: ItemImpl = parse2(nullable).expect("array_type_impl");

        quote!(
            #non_nullable
            #nullable
        )
    }

    /// Add an `StructArrayType` implementation for the derive input.
    fn struct_array_type_impl(&self) -> ItemImpl {
        let narrow = util::narrow();

        // Generics
        let mut generics = self.generics.clone();
        AddTypeParamBoundWithSelf(Struct::array_type_bound()).visit_generics_mut(&mut generics);
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        // Array generics
        let mut generics = generics.clone();
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        let (_, array_ty_generics, _) = generics.split_for_impl();

        let ident = self.ident;
        let array_struct_ident = self.array_struct_ident();
        let tokens = quote! {
            impl #impl_generics #narrow::array::StructArrayType for #ident #ty_generics #where_clause {
                type Array<Buffer: #narrow::buffer::BufferType> = #array_struct_ident #array_ty_generics;
            }
        };
        parse2(tokens).expect("struct_array_type_impl")
    }

    /// Add an `StructArrayTypeFields` implementation for the derive input.
    #[cfg(feature = "arrow-rs")]
    fn struct_array_type_fields_impl(&self) -> ItemImpl {
        let narrow = util::narrow();

        // Generics
        let mut generics = self.generics.clone();
        SelfReplace::new(self.ident, &generics).visit_generics_mut(&mut generics);
        AddTypeParamBoundWithSelf(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        generics
            .make_where_clause()
            .predicates
            .extend(self.where_predicate_fields(parse_quote!(#narrow::arrow::Array)));
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let ident = self.array_struct_ident();
        let field_name = self.ident.to_string();
        let tokens = if matches!(self.fields, Fields::Unit) {
            quote!(impl #impl_generics #narrow::arrow::StructArrayTypeFields for #ident #ty_generics #where_clause {
                const NAMES: &'static [&'static str] = &[#field_name];
                fn fields() -> #narrow::arrow_schema::Fields {
                    #narrow::arrow_schema::Fields::from([
                        ::std::sync::Arc::new(#narrow::arrow_schema::Field::new(#field_name, #narrow::arrow_schema::DataType::Null, true)),
                    ])
                }
            })
        } else {
            // Fields
            let field_ident = self.field_idents().map(|ident| ident.to_string());
            let field_name = field_ident.clone();
            let field_ty = self.field_types();
            let field_ty_drop = self.field_types_drop_option();
            let fields = quote!(
                #(
                    ::std::sync::Arc::new(<<#field_ty as ::narrow::array::ArrayType<#field_ty_drop>>::Array<Buffer, #narrow::offset::NA, #narrow::array::union::NA> as #narrow::arrow::Array>::as_field(#field_ident)),
                )*
            );
            quote! {
                impl #impl_generics #narrow::arrow::StructArrayTypeFields for #ident #ty_generics #where_clause {
                    const NAMES: &'static [&'static str] = &[
                        #(
                            #field_name,
                        )*
                    ];
                    fn fields() -> #narrow::arrow_schema::Fields {
                        #narrow::arrow_schema::Fields::from([
                            #fields
                        ])
                    }
                }
            }
        };

        parse2(tokens).expect("struct_array_type_fields_impl")
    }

    /// Add an `Into` implementation for the array to convert to a vec of array refs
    #[cfg(feature = "arrow-rs")]
    fn struct_array_into_array_refs(&self) -> ItemImpl {
        let narrow = util::narrow();

        // Generics
        let mut generics = self.generics.clone();
        SelfReplace::new(self.ident, &generics).visit_generics_mut(&mut generics);
        AddTypeParamBoundWithSelf(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        generics
            .make_where_clause()
            .predicates
            .extend(self.where_predicate_fields_arrow_array_into());
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        // Fields
        let field_arrays = match self.fields {
            Fields::Named(_) => {
                let field_ident = self.field_idents();
                quote!(
                    #(
                        value.#field_ident.into(),
                    )*
                )
            }
            Fields::Unnamed(_) => {
                let field_idx = self
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| Index::from(idx));
                quote!(
                    #(
                        value.#field_idx.into(),
                    )*
                )
            }
            Fields::Unit => {
                quote!(value.0.into())
            }
        };

        let ident = self.array_struct_ident();
        let tokens = quote! {
            impl #impl_generics ::std::convert::From<#ident #ty_generics> for ::std::vec::Vec<::std::sync::Arc<dyn #narrow::arrow_array::Array>> #where_clause  {
                fn from(value: #ident #ty_generics) -> Self {
                    vec![
                        #field_arrays
                    ]
                }
            }
        };
        parse2(tokens).expect("struct_array_into_array_refs")
    }

    /// Add an `From` implementation for the array to convert from a vec of array refs
    #[cfg(feature = "arrow-rs")]
    fn struct_array_from_array_refs(&self) -> ItemImpl {
        let narrow = util::narrow();

        // Generics
        let mut generics = self.generics.clone();
        SelfReplace::new(self.ident, &generics).visit_generics_mut(&mut generics);
        AddTypeParamBoundWithSelf(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        generics
            .make_where_clause()
            .predicates
            .extend(self.where_predicate_fields(parse_quote!(
                ::std::convert::From<::std::sync::Arc<dyn #narrow::arrow_array::Array>>
            )));
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        // Fields
        let field_arrays = self.surround_with_delimiters(match self.fields {
            Fields::Named(_) => {
                let field_ident = self.field_idents();
                quote!(
                    #(
                        #field_ident: arrays.next().expect("array").into(),
                    )*
                )
            }
            Fields::Unnamed(_) => {
                let field = std::iter::repeat(quote!(arrays.next().expect("array").into()))
                    .take(self.fields.len());
                quote!(
                    #(
                        #field,
                    )*
                )
            }
            Fields::Unit => {
                quote!(arrays.next().expect("array").into())
            }
        });
        let ident = self.array_struct_ident();
        let tokens = quote! {
            impl #impl_generics ::std::convert::From<::std::vec::Vec<::std::sync::Arc<dyn #narrow::arrow_array::Array>>> for #ident #ty_generics #where_clause  {
                fn from(value: ::std::vec::Vec<::std::sync::Arc<dyn #narrow::arrow_array::Array>>) -> Self {
                    let mut arrays = value.into_iter();
                    let result = Self #field_arrays;
                    assert!(arrays.next().is_none());
                    result
                }
            }
        };
        parse2(tokens).expect("struct_array_from_array_refs")
    }

    /// Returns the struct definition of the Array wrapper struct.
    fn array_struct_def(&self) -> ItemStruct {
        let narrow = util::narrow();

        // Generics
        let mut generics = self.generics.clone();
        SelfReplace::new(self.ident, &generics).visit_generics_mut(&mut generics);
        AddTypeParamBoundWithSelf(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        let (impl_generics, _, where_clause) = generics.split_for_impl();

        // Fields
        let fields = self.surround_with_delimiters(match self.fields {
            Fields::Named(_) => {
                let field_ident = self.field_idents();
                let field_ty = self.field_types();
                let field_vis = self.field_vis();
                let field_ty_drop = self.field_types_drop_option();
                quote!(
                    #(
                        #field_vis #field_ident: <#field_ty as #narrow::array::ArrayType<#field_ty_drop>>::Array<Buffer, #narrow::offset::NA, #narrow::array::union::NA>,
                    )*
                )
            }
            Fields::Unnamed(_) => {
                let field_ty = self.field_types();
                let field_vis = self.field_vis();
                let field_ty_drop = self.field_types_drop_option();
                quote!(
                    #(
                        #field_vis <#field_ty as #narrow::array::ArrayType<#field_ty_drop>>::Array<Buffer, #narrow::offset::NA, #narrow::array::union::NA>,
                    )*
                )
            }
            Fields::Unit => {
                let ident = self.ident;
                // We use the visibility of the item for the inner null array.
                let vis = self.vis;
                let (_, ty_generics, _) = self.generics.split_for_impl();
                quote!(#vis #narrow::array::NullArray<#ident #ty_generics, #narrow::NonNullable, Buffer>)
            }
        });

        let rest = if matches!(self.fields, Fields::Named(_)) {
            quote!(#where_clause #fields)
        } else {
            quote!(#fields #where_clause;)
        };

        let vis = self.vis;
        let ident = self.array_struct_ident();

        let tokens = quote!(
            #vis struct #ident #impl_generics #rest
        );
        parse2(tokens).expect("array_struct_def")
    }

    fn array_clone_impl(&self) -> ItemImpl {
        let narrow = util::narrow();

        // Generics
        let mut generics = self.generics.clone();
        SelfReplace::new(self.ident, &generics).visit_generics_mut(&mut generics);
        AddTypeParamBoundWithSelf(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        generics
            .make_where_clause()
            .predicates
            .extend(self.where_predicate_fields(parse_quote!(::std::clone::Clone)));
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let clone_fields = self.surround_with_delimiters(match self.fields {
            Fields::Named(_) => {
                let field_ident = self.field_idents();
                quote!(
                    #(
                        #field_ident: self.#field_ident.clone(),
                    )*
                )
            }
            Fields::Unnamed(_) => {
                let field_idx = self
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| Index::from(idx));
                quote!(
                    #(
                        self.#field_idx.clone(),
                    )*
                )
            }
            Fields::Unit => {
                quote!(self.0.clone())
            }
        });

        let ident = self.array_struct_ident();
        let tokens = quote!(
            impl #impl_generics ::std::clone::Clone for #ident #ty_generics #where_clause {
                fn clone(&self) -> Self {
                    Self #clone_fields
                }
            }
        );
        parse2(tokens).expect("array_clone_impl")
    }

    fn array_default_impl(&self) -> ItemImpl {
        let narrow = util::narrow();

        // Generics
        let mut generics = self.generics.clone();
        SelfReplace::new(self.ident, &generics).visit_generics_mut(&mut generics);
        AddTypeParamBoundWithSelf(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        generics
            .make_where_clause()
            .predicates
            .extend(self.where_predicate_fields(parse_quote!(::std::default::Default)));
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let default_fields = self.surround_with_delimiters(match self.fields {
            Fields::Named(_) => {
                let field_ident = self.field_idents();
                quote!(
                    #(
                        #field_ident: ::std::default::Default::default(),
                    )*
                )
            }
            Fields::Unnamed(_) => {
                let default_field = std::iter::repeat(quote!(::std::default::Default::default()))
                    .take(self.fields.len());
                quote!(
                    #(
                        #default_field,
                    )*
                )
            }
            Fields::Unit => {
                quote!(::std::default::Default::default())
            }
        });

        let ident = self.array_struct_ident();
        let tokens = quote!(
            impl #impl_generics ::std::default::Default for #ident #ty_generics #where_clause {
                fn default() -> Self {
                    Self #default_fields
                }
            }
        );
        parse2(tokens).expect("array_default_impl")
    }

    fn array_len_impl(&self) -> ItemImpl {
        let narrow = util::narrow();

        // Generics
        let mut generics = self.generics.clone();
        SelfReplace::new(self.ident, &generics).visit_generics_mut(&mut generics);
        AddTypeParamBoundWithSelf(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        // For the impl it would also work to just have a Length bound of the first field.
        generics
            .make_where_clause()
            .predicates
            .extend(self.where_predicate_fields(parse_quote!(#narrow::Length)));
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let ident = self.array_struct_ident();
        let len = match self.fields {
            Fields::Named(_) => {
                let field_ident = self.field_idents().next().unwrap();
                quote!(self.#field_ident.len())
            }
            Fields::Unnamed(_) | Fields::Unit => {
                quote!(self.0.len())
            }
        };
        let tokens = quote!(
            impl #impl_generics #narrow::Length for #ident #ty_generics #where_clause {
                fn len(&self) -> usize {
                    #len
                }
            }
        );
        parse2(tokens).expect("array_len_impl")
    }

    fn array_extend_impl(&self) -> ItemImpl {
        let narrow = util::narrow();
        let ident = self.ident;

        // Generics
        let mut ident_generics = self.generics.clone();
        SelfReplace::new(ident, &ident_generics).visit_generics_mut(&mut ident_generics);
        let (_, ident_ty_generics, _) = ident_generics.split_for_impl();

        // Array generics
        let mut generics = self.generics.clone();
        AddTypeParamBoundWithSelf(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        generics
            .make_where_clause()
            .predicates
            .extend(
                self.field_types().zip(self.field_types_drop_option())
                    .map::<WherePredicate, _>(move |(ty, ty_drop)| parse_quote!(<#ty as #narrow::array::ArrayType<#ty_drop>>::Array<Buffer, #narrow::offset::NA, #narrow::array::union::NA>: ::std::iter::Extend<#ty>))
            );
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let array_struct_ident = self.array_struct_ident();
        let extend = match self.fields {
            Fields::Unnamed(_) => {
                let field_ident = self.field_idents().collect::<Vec<_>>();
                let fields = self.surround_with_delimiters(quote!(#( #field_ident, )*));
                let field_idx = self
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| Index::from(idx));
                quote!(
                    iter.into_iter().for_each(|#ident #fields | {
                        #(
                            self.#field_idx.extend(::std::iter::once(#field_ident));
                        )*
                    });
                )
            }
            Fields::Named(_) => {
                let field_ident = self.field_idents().collect::<Vec<_>>();
                let fields = self.surround_with_delimiters(quote!(#( #field_ident, )*));
                quote!(
                    iter.into_iter().for_each(|#ident #fields | {
                        #(
                            self.#field_ident.extend(::std::iter::once(#field_ident));
                        )*
                    });
                )
            }
            Fields::Unit => quote!(self.0.extend(iter)),
        };
        let tokens = quote!(
            impl #impl_generics ::std::iter::Extend<#ident #ident_ty_generics> for #array_struct_ident #ty_generics #where_clause {
                fn extend<_I: ::std::iter::IntoIterator<Item = #ident #ident_ty_generics>>(&mut self, iter: _I) {
                    #extend
                }
            }
        );
        parse2(tokens).expect("array_extend_impl")
    }

    fn array_from_iter_impl(&self) -> ItemImpl {
        let narrow = util::narrow();
        let ident = self.ident;

        // Generics
        let mut ident_generics = self.generics.clone();
        SelfReplace::new(ident, &ident_generics).visit_generics_mut(&mut ident_generics);
        let (_, ident_ty_generics, _) = ident_generics.split_for_impl();

        // Array generics
        let mut generics = self.generics.clone();
        AddTypeParamBoundWithSelf(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        generics
            .make_where_clause()
            .predicates
            .extend(
                self.field_types().zip(self.field_types_drop_option())
                    .map::<WherePredicate, _>(move |(ty, ty_drop)| parse_quote!(<#ty as #narrow::array::ArrayType<#ty_drop>>::Array<Buffer, #narrow::offset::NA, #narrow::array::union::NA>: ::std::default::Default + ::std::iter::Extend<#ty>))
            );
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let array_struct_ident = self.array_struct_ident();
        let from_iter = match self.fields {
            Fields::Unnamed(_) => {
                let field_ident = self.field_idents().collect::<Vec<_>>();
                let fields = self.surround_with_delimiters(quote!(#( #field_ident, )*));
                let tuple = self.field_tuple();
                quote!(
                    let #tuple = iter.into_iter().map(|#ident #fields| #tuple).unzip();
                    Self (
                        #(
                            #field_ident,
                        )*
                    )
                )
            }
            Fields::Named(_) => {
                let field_ident = self.field_idents().collect::<Vec<_>>();
                let fields = self.surround_with_delimiters(quote!(#( #field_ident, )*));
                let tuple = self.field_tuple();
                quote!(
                    let #tuple = iter.into_iter().map(|#ident #fields| #tuple).unzip();
                    Self {
                        #(
                            #field_ident,
                        )*
                    }
                )
            }
            Fields::Unit => quote!(Self(iter.into_iter().collect())),
        };
        let tokens = quote!(
            impl #impl_generics ::std::iter::FromIterator<#ident #ident_ty_generics> for #array_struct_ident #ty_generics #where_clause {
                fn from_iter<_I: ::std::iter::IntoIterator<Item = #ident #ident_ty_generics>>(iter: _I) -> Self {
                    #from_iter
                }
            }
        );
        parse2(tokens).expect("array_from_iter_impl")
    }

    fn array_iter_struct_def(&self) -> ItemStruct {
        let narrow = util::narrow();

        // Array generics
        let mut generics = self.generics.clone();
        AddTypeParamBoundWithSelf(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        generics
            .make_where_clause()
            .predicates
            .extend(
                self.field_types().zip(self.field_types_drop_option())
                    .map::<WherePredicate, _>(move |(ty, ty_drop)| parse_quote!(<#ty as #narrow::array::ArrayType<#ty_drop>>::Array<Buffer, #narrow::offset::NA, #narrow::array::union::NA>: ::std::iter::IntoIterator<Item = #ty>))
            );
        let (impl_generics, _, where_clause) = generics.split_for_impl();

        // Iter struct definition
        let array_iter_struct_ident = self.array_iter_struct_ident();
        let fields = self.surround_with_delimiters(match self.fields {
            Fields::Unnamed(_) => {
                let field_ty = self.field_types();
                let field_vis = self.field_vis();
                let field_ty_drop = self.field_types_drop_option();
                let narrow = util::narrow();
                quote!(
                    #(
                        #field_vis <<#field_ty as #narrow::array::ArrayType<#field_ty_drop>>::Array<Buffer, #narrow::offset::NA, #narrow::array::union::NA> as ::std::iter::IntoIterator>::IntoIter,
                    )*
                )
            }
            Fields::Named(_) => {
                let field_ident = self.field_idents();
                let field_ty = self.field_types();
                let field_vis = self.field_vis();
                let field_ty_drop = self.field_types_drop_option();
                let narrow = util::narrow();
                quote!(
                    #(
                        #field_vis #field_ident: <<#field_ty as #narrow::array::ArrayType<#field_ty_drop>>::Array<Buffer, #narrow::offset::NA, #narrow::array::union::NA> as ::std::iter::IntoIterator>::IntoIter,
                    )*
                )
            }
            Fields::Unit => {
                let ident = self.ident;
                let narrow = util::narrow();
                let mut generics = generics.clone();
                let (_, ty_generics, _) = self.generics.split_for_impl();
                generics
                    .make_where_clause()
                    .predicates
                    .extend(
                       std::iter::once::<WherePredicate>(parse_quote!(#narrow::array::NullArray<#ident #ty_generics, #narrow::NonNullable, Buffer>: ::std::iter::IntoIterator<Item = #ident #ty_generics>))
                    );
                let (impl_generics, _, where_clause) = generics.split_for_impl();
                let narrow = util::narrow();
                let vis = self.vis;
                let tokens = quote!(
                    #vis struct #array_iter_struct_ident #impl_generics(
                        #vis <#narrow::array::NullArray<#ident #ty_generics, #narrow::NonNullable, Buffer> as IntoIterator>::IntoIter
                    ) #where_clause;
                );
                return parse2(tokens).expect("array_iter_struct_def");
            }
        });
        let vis = self.vis;

        let rest = if matches!(self.fields, Fields::Named(_)) {
            quote!(#where_clause #fields)
        } else {
            quote!(#fields #where_clause;)
        };

        let tokens = quote! {
            #vis struct #array_iter_struct_ident #impl_generics #rest
        };
        parse2(tokens).expect("array_iter_struct_def")
    }

    fn array_iter_iterator_impl(&self) -> ItemImpl {
        let narrow = util::narrow();

        // Array generics
        let mut generics = self.generics.clone();
        AddTypeParamBoundWithSelf(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        generics
            .make_where_clause()
            .predicates
            .extend(
                self.field_types().zip(self.field_types_drop_option())
                    .map::<WherePredicate, _>(move |(ty, ty_drop)| parse_quote!(<#ty as #narrow::array::ArrayType<#ty_drop>>::Array<Buffer, #narrow::offset::NA, #narrow::array::union::NA>: ::std::iter::IntoIterator<Item = #ty>))
            );
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let next = match self.fields {
            Fields::Unit => quote!(self.0.next()),
            Fields::Unnamed(_) => {
                let mut field_idx = self
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| Index::from(idx));
                let first = field_idx.next().unwrap();
                let ident = self.ident;
                quote!(
                    self.#first.next().map(|first| {
                        #ident(
                            first,
                            #(
                                self.#field_idx.next().unwrap(),
                            )*
                        )
                    })
                )
            }
            Fields::Named(_) => {
                let field_ident = self.field_idents().skip(1);
                let first = self.field_idents().nth(0);
                let ident = self.ident;
                quote!(
                    self.#first.next().map(|#first| {
                        #ident{
                            #first,
                            #(
                                #field_ident: self.#field_ident.next().unwrap(),
                            )*
                        }
                    })
                )
            }
        };

        let ident = self.ident;
        let array_iter_struct_ident = self.array_iter_struct_ident();
        let (_, ty_generics_item, _) = self.generics.split_for_impl();
        let tokens = quote! {
            impl #impl_generics ::std::iter::Iterator for #array_iter_struct_ident #ty_generics #where_clause {
                type Item = #ident #ty_generics_item;

                fn next(&mut self) -> Option<Self::Item> {
                    #next
                }
            }
        };
        parse2(tokens).expect("array_iter_iterator_impl")
    }

    fn array_into_iter_impl(&self) -> ItemImpl {
        let narrow = util::narrow();
        let ident = self.ident;

        // Generics
        let mut ident_generics = self.generics.clone();
        SelfReplace::new(ident, &ident_generics).visit_generics_mut(&mut ident_generics);

        // Array generics
        let mut generics = self.generics.clone();
        AddTypeParamBoundWithSelf(Self::array_type_bound()).visit_generics_mut(&mut generics);
        AddTypeParam(parse_quote!(Buffer: #narrow::buffer::BufferType))
            .visit_generics_mut(&mut generics);
        generics
            .make_where_clause()
            .predicates
            .extend(
                self.field_types().zip(self.field_types_drop_option())
                    .map::<WherePredicate, _>(move |(ty, ty_drop)| parse_quote!(<#ty as #narrow::array::ArrayType<#ty_drop>>::Array<Buffer, #narrow::offset::NA, #narrow::array::union::NA>: ::std::iter::IntoIterator<Item = #ty>))
            );
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        // Iter struct definition
        // Trait impl
        let array_iter_struct_ident = self.array_iter_struct_ident();
        let array_struct_ident = self.array_struct_ident();
        let fields = self.surround_with_delimiters(match self.fields {
            Fields::Unnamed(_) => {
                let field_idx = self
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| Index::from(idx));
                quote!(
                    #(
                        self.#field_idx.into_iter(),
                    )*
                )
            }
            Fields::Named(_) => {
                let field_ident = self.field_idents();
                quote!(
                    #(
                        #field_ident: self.#field_ident.into_iter(),
                    )*
                )
            }
            Fields::Unit => quote!(self.0.into_iter()),
        });
        let (_, ty_generics_item, _) = self.generics.split_for_impl();
        let tokens = quote!(
            impl #impl_generics ::std::iter::IntoIterator for #array_struct_ident #ty_generics #where_clause {
                type Item = #ident #ty_generics_item;
                type IntoIter = #array_iter_struct_ident #ty_generics;
                fn into_iter(self) -> Self::IntoIter {
                    #array_iter_struct_ident #fields
                }
            }
        );
        parse2(tokens).expect("array_into_iter_impl")
    }

    fn field_tuple(&self) -> TokenStream {
        let mut ident = self.field_idents();
        let initial = ident.next_back().map(|last| quote!((#last, ()))).unwrap();
        ident.rfold(initial, |acc, x| quote!((#x, #acc)))
    }

    fn where_predicate_fields(
        &self,
        bound: TypeParamBound,
    ) -> impl Iterator<Item = WherePredicate> + '_ {
        let narrow = util::narrow();
        self.field_types().zip(self.field_types_drop_option())
            .map(move |(ty, ty_drop)| parse_quote!(<#ty as #narrow::array::ArrayType<#ty_drop>>::Array<Buffer, #narrow::offset::NA, #narrow::array::union::NA>: #bound))
    }

    #[cfg(feature = "arrow-rs")]
    fn where_predicate_fields_arrow_array_into(&self) -> impl Iterator<Item = WherePredicate> + '_ {
        self.field_types().zip(self.field_types_drop_option())
            .map(move |(ty, ty_drop)| {
                let narrow = util::narrow();
                parse_quote!(
                <#ty as #narrow::array::ArrayType<#ty_drop>>::Array<Buffer, #narrow::offset::NA, #narrow::array::union::NA>:
                    ::std::convert::Into<
                        ::std::sync::Arc<dyn #narrow::arrow_array::Array>
                    >
            )})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surround() {
        // unit
        let input = quote!(
            struct Foo;
        );
        let derive_input: syn::DeriveInput = parse2(input.clone()).unwrap();
        let as_struct: syn::ItemStruct = parse2(input).unwrap();

        assert_eq!(
            Struct::new(&derive_input, &as_struct.fields)
                .surround_with_delimiters(quote!(x))
                .to_string(),
            "(x)"
        );

        // unnamed
        let input = quote!(
            struct Foo(u32, u8);
        );
        let derive_input: syn::DeriveInput = parse2(input.clone()).unwrap();
        let as_struct: syn::ItemStruct = parse2(input).unwrap();

        assert_eq!(
            Struct::new(&derive_input, &as_struct.fields)
                .surround_with_delimiters(quote!(x))
                .to_string(),
            "(x)"
        );

        // named
        let input = quote!(
            struct Foo {
                a: u32,
                b: u8,
            }
        );
        let derive_input: syn::DeriveInput = parse2(input.clone()).unwrap();
        let as_struct: syn::ItemStruct = parse2(input).unwrap();

        assert_eq!(
            Struct::new(&derive_input, &as_struct.fields)
                .surround_with_delimiters(quote!(x))
                .to_string(),
            "{ x }"
        );
    }
}
