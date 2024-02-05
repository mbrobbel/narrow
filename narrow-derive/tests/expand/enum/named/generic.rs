#[derive(narrow_derive::ArrayType, Clone, Copy)]
enum Foo<T> {
    Foo { bar: T },
    Bar(T),
    None,
}
