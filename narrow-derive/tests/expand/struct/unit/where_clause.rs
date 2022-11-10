#[derive(narrow_derive::Array)]
pub(super) struct Foo<const N: bool = false>
where
    Self: Sized;
