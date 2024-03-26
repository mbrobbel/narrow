enum Foo<T> {
    Foo { bar: T },
    Bar(T),
    None,
}
#[automatically_derived]
impl<T: ::core::clone::Clone> ::core::clone::Clone for Foo<T> {
    #[inline]
    fn clone(&self) -> Foo<T> {
        match self {
            Foo::Foo { bar: __self_0 } => {
                Foo::Foo {
                    bar: ::core::clone::Clone::clone(__self_0),
                }
            }
            Foo::Bar(__self_0) => Foo::Bar(::core::clone::Clone::clone(__self_0)),
            Foo::None => Foo::None,
        }
    }
}
#[automatically_derived]
impl<T: ::core::marker::Copy> ::core::marker::Copy for Foo<T> {}
