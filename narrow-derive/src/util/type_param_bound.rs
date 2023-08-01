use syn::{visit_mut::VisitMut, TypeParam, TypeParamBound};

/// Adds the owned type param bound to all type parameters.
pub struct AddTypeParamBound(pub TypeParamBound);

impl VisitMut for AddTypeParamBound {
    fn visit_type_param_mut(&mut self, type_param: &mut TypeParam) {
        type_param.bounds.push(self.0.clone());
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
}
