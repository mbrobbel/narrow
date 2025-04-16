use syn::{TypeParam, TypeParamBound, parse_quote, visit_mut::VisitMut};

/// Adds the owned type param bound to all type parameters.
pub struct AddTypeParamBound(pub TypeParamBound);

impl VisitMut for AddTypeParamBound {
    fn visit_type_param_mut(&mut self, type_param: &mut TypeParam) {
        type_param.bounds.push(self.0.clone());
    }
}

/// Adds the owned type param bound to all type parameters.
pub struct AddTypeParamBoundWithSelf(pub TypeParamBound);

impl VisitMut for AddTypeParamBoundWithSelf {
    fn visit_type_param_mut(&mut self, type_param: &mut TypeParam) {
        let ident = type_param.ident.clone();
        let bound = self.0.clone();
        type_param.bounds.push(parse_quote!(#bound < #ident >));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn type_param_bound() {
        let mut item: syn::ItemStruct = parse_quote!(
            struct Foo<T>(T)
            where
                T: Y;
        );
        AddTypeParamBound(parse_quote!(X)).visit_item_struct_mut(&mut item);
        assert_eq!(
            quote!(#item).to_string(),
            "struct Foo < T : X > (T) where T : Y ;"
        );
    }

    #[test]
    fn type_param_bound_with() {
        let mut item: syn::ItemStruct = parse_quote!(
            struct Foo<T>(T)
            where
                T: Y;
        );
        AddTypeParamBoundWithSelf(parse_quote!(X)).visit_item_struct_mut(&mut item);
        assert_eq!(
            quote!(#item).to_string(),
            "struct Foo < T : X < T > > (T) where T : Y ;"
        );
    }
}
