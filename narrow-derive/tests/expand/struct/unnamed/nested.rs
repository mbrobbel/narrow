#[derive(narrow_derive::ArrayType)]
struct Foo(u32);

#[derive(narrow_derive::ArrayType)]
struct Bar(Foo);
