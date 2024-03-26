enum FooBar {
    Unit,
    Foo { bar: u32 },
    Bar { foo: bool },
    FooBar { foo: String, bar: Option<u8> },
}
