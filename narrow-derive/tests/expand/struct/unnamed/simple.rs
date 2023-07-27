#[derive(narrow_derive::ArrayType)]
struct Foo(u32);

#[derive(narrow_derive::ArrayType)]
struct Bar(u8, u16, u32, u64);

#[derive(narrow_derive::ArrayType)]
struct FooBar<T>(T);
