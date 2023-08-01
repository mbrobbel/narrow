#[derive(narrow_derive::ArrayType)]
struct Foo<'a, T: Add<Self>>(&'a T)
where
    Self: Sized,
    <T as Add<Self>>::Output: Debug;

#[derive(narrow_derive::ArrayType)]
struct FooBar<T>(T);
