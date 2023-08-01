#[derive(narrow_derive::ArrayType)]
struct Foo {
    a: u32,
    b: bool,
    c: Option<Vec<u8>>,
}
