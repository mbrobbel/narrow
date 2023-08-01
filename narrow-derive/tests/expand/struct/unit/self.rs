#[derive(narrow_derive::ArrayType)]
struct Foo
where
    Self: Debug;
