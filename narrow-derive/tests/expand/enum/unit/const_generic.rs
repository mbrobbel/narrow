#[derive(narrow_derive::ArrayType)]
enum FooBar<const X: bool> {
    Foo,
    Bar,
}
