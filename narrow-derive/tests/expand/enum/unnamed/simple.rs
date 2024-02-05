#[derive(narrow_derive::ArrayType)]
enum FooBar {
    Foo(bool),
    Bar(u8, u16),
}
