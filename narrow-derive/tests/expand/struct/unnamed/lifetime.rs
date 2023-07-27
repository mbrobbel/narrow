#[derive(narrow_derive::ArrayType)]
struct Foo<'a, T>(&'a T);
