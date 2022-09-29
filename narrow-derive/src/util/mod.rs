use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_quote, visit::Visit, visit_mut::VisitMut, Generics, Ident, Type, TypePath, WherePredicate,
};

use crate::NARROW;

mod bounds;

/// Returns the name of the `narrow` crate. Panics when the `narrow` crate is
/// not found.
pub(super) fn narrow() -> TokenStream {
    let ident = format_ident!("{}", &*NARROW);
    quote!(#ident)
}

pub(super) fn raw_array(ident: &Ident) -> (Ident, String) {
    (
        format_ident!("Raw{}Array", ident),
        format!(" Array with [{ident}] values."),
    )
}

// pub(super) fn alias_array_ident(ident: &Ident) -> Ident {
//     format_ident!("{}Array", ident)
// }

/// Replace Self with ident in where clauses.
pub(super) struct SelfReplace<'a> {
    ident: &'a Ident,
}

impl SelfReplace<'_> {
    // pub(super) fn generics(ident: &Ident, generics: &Generics) -> Generics {
    //     let mut generics = generics.clone();
    //     SelfReplace { ident }.visit_generics_mut(&mut generics);
    //     generics
    // }
}

impl Visit<'_> for SelfReplace<'_> {
    fn visit_generics(&mut self, generics: &Generics) {
        let mut generics = generics.clone();
        self.visit_generics_mut(&mut generics);
    }
}

// todo(mb): also replace self in bounds
impl VisitMut for SelfReplace<'_> {
    fn visit_generics_mut(&mut self, generics: &mut Generics) {
        let ty_generics = quote!(generics.split_for_impl().1);
        generics
            .make_where_clause()
            .predicates
            .iter_mut()
            .for_each(|predicate| {
                if let WherePredicate::Type(predicate_type) = predicate {
                    if let Type::Path(TypePath { path, .. }) = &predicate_type.bounded_ty {
                        if path == &parse_quote!(Self) {
                            let ident = self.ident;
                            predicate_type.bounded_ty = parse_quote!(#ident #ty_generics);
                        }
                    }
                }
            });
    }
}

pub(super) struct AddNullableConstGeneric;

impl VisitMut for AddNullableConstGeneric {
    fn visit_generics_mut(&mut self, generics: &mut Generics) {
        generics.params.push(parse_quote!(const N: bool));
    }
}

pub(super) struct NullableConstGeneric;

impl NullableConstGeneric {
    pub(super) fn ident() -> Ident {
        format_ident!("_NARROW_NULLABILITY")
    }
}

impl NullableConstGeneric {
    // pub(super) fn generics(generics: &Generics) -> Generics {
    //     let mut generics = generics.clone();
    //     NullableConstGeneric.visit_generics_mut(&mut generics);
    //     generics
    // }
}

impl VisitMut for NullableConstGeneric {
    fn visit_generics_mut(&mut self, generics: &mut Generics) {
        let ident = Self::ident();
        generics
            .params
            .push(parse_quote!(const #ident: bool = false));
    }
}
