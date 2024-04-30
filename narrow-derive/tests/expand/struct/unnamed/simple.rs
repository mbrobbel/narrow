#[derive(narrow_derive::ArrayType)]
struct Foo<T: Sized>(T, u32);
