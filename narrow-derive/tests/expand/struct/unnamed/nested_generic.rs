#[derive(narrow_derive::ArrayType)]
struct Foo<T>(T)
where
    T: Copy;

#[derive(narrow_derive::ArrayType)]
struct Bar<'a, T>(&'a Foo<T>);

#[derive(narrow_derive::ArrayType)]
struct FooBar<'a>(Bar<'a, u32>);
