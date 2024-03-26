enum FooBar<T: Default> {
    Foo,
    Bar(T),
    FooBar { foo_bar: T },
}
