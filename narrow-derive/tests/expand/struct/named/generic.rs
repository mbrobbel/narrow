#[derive(narrow_derive::ArrayType)]
struct Foo<'a, T>
where
    T: Copy,
{
    a: &'a T,
}
