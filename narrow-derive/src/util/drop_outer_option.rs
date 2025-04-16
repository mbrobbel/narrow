use syn::{
    AngleBracketedGenericArguments, GenericArgument, PathArguments, Type, TypePath,
    visit_mut::VisitMut,
};

/// Adds the owned type param to all type parameters.
pub struct DropOuterParam;

impl VisitMut for DropOuterParam {
    fn visit_type_mut(&mut self, i: &mut syn::Type) {
        if let Type::Path(TypePath { path, .. }) = i {
            if let Some(path_segment) = path
                .segments
                .iter()
                .find(|path_segment| "Option" == path_segment.ident.to_string().as_str())
            {
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    ref args,
                    ..
                }) = path_segment.arguments
                {
                    if let Some(GenericArgument::Type(ty)) = args.first() {
                        // Replace with the inner type of the option
                        *i = ty.clone();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn drop_outer_option() {
        let mut ty: syn::Type = parse_quote!(Foo);
        DropOuterParam.visit_type_mut(&mut ty);
        assert_eq!(quote!(#ty).to_string(), "Foo");

        let mut ty: syn::Type = parse_quote!(Option<Foo>);
        DropOuterParam.visit_type_mut(&mut ty);
        assert_eq!(quote!(#ty).to_string(), "Foo");

        let mut ty: syn::Type = parse_quote!(option::Option<Foo>);
        DropOuterParam.visit_type_mut(&mut ty);
        assert_eq!(quote!(#ty).to_string(), "Foo");

        let mut ty: syn::Type = parse_quote!(std::option::Option<Foo>);
        DropOuterParam.visit_type_mut(&mut ty);
        assert_eq!(quote!(#ty).to_string(), "Foo");

        let mut ty: syn::Type = parse_quote!(::std::option::Option<Foo>);
        DropOuterParam.visit_type_mut(&mut ty);
        assert_eq!(quote!(#ty).to_string(), "Foo");

        let mut ty: syn::Type = parse_quote!(Option<[Foo; 3]>);
        DropOuterParam.visit_type_mut(&mut ty);
        assert_eq!(quote!(#ty).to_string(), "[Foo ; 3]");

        let mut ty: syn::Type = parse_quote!(Option<Option<Foo>>);
        DropOuterParam.visit_type_mut(&mut ty);
        assert_eq!(quote!(#ty).to_string(), "Option < Foo >");
    }
}
