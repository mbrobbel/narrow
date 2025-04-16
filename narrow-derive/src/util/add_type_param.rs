use syn::{GenericParam, Generics, TypeParam, visit_mut::VisitMut};

/// Adds the owned type param to all type parameters.
pub struct AddTypeParam(pub TypeParam);

impl VisitMut for AddTypeParam {
    fn visit_generics_mut(&mut self, generics: &mut Generics) {
        generics.params.push(GenericParam::Type(self.0.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn type_param() {
        let mut item: syn::ItemStruct = parse_quote!(
            struct Foo<T>(T);
        );
        AddTypeParam(parse_quote!(X)).visit_item_struct_mut(&mut item);
        assert_eq!(quote!(#item).to_string(), "struct Foo < T , X > (T) ;");
    }
}
