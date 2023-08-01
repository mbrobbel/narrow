use syn::{parse_quote, visit_mut::VisitMut, Generics, Ident, PathSegment};

/// Replace `Self` in generics with `ident`.
///
/// When generating additional helper types as part of the derived implementation
/// they may refer to the type of the derive input using `Self`.
/// These helper types replace `Self` with `ident` here to reflect the intended
/// bound.
pub(crate) struct SelfReplace(PathSegment);

impl SelfReplace {
    pub(crate) fn new(ident: &Ident, generics: &Generics) -> Self {
        let (_, ty_generics, _) = generics.split_for_impl();
        Self(parse_quote!(#ident #ty_generics))
    }
}

impl VisitMut for SelfReplace {
    fn visit_path_segment_mut(&mut self, path_segment: &mut PathSegment) {
        if path_segment.ident == "Self" {
            // Note: the `Self` type doesn't accept type parameters,
            // so we can ignore path arguments.
            *path_segment = self.0.clone();
        } else {
            self.visit_path_arguments_mut(&mut path_segment.arguments);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn self_replace() {
        let mut item: syn::ItemStruct = parse_quote!(
            struct Foo<T, U: Sub<Self>>
            where
                U: Add<Self>,
                Self: X,
                <U as Add<Self>>::Output: X,
                T: IntoIterator<Item = Self>;
        );
        SelfReplace::new(&item.ident, &item.generics).visit_item_struct_mut(&mut item);
        assert_eq!(
            quote!(#item).to_string(),
            "struct Foo < T , U : Sub < Foo < T , U > > > \
             where \
               U : Add < Foo < T , U > > , \
               Foo < T , U > : X , \
               < U as Add < Foo < T , U > > > :: Output : X , \
               T : IntoIterator < Item = Foo < T , U > > ;"
        );
    }
}
