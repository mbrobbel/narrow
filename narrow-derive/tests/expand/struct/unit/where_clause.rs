#[derive(narrow_derive::ArrayType)]
pub(super) struct Foo<const N: bool = false>
where
    Self: Sized,
    (): From<Self>;
