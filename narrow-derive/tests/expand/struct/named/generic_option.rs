#[derive(narrow_derive::ArrayType)]
struct Bar<T> {
    a: u32,
    b: Option<bool>,
    c: Option<T>,
}
