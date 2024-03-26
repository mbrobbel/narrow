enum FooBar<T>
where
    T: Default,
    Self: Clone,
{
    Foo,
    Bar(T),
    FooBar { foo_bar: T },
}
