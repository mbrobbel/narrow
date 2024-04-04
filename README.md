![Narrow logo](https://raw.githubusercontent.com/mbrobbel/narrow/main/narrow.svg)

[![crates.io](https://img.shields.io/crates/v/narrow.svg)](https://crates.io/crates/narrow)
[![docs.rs](https://docs.rs/narrow/badge.svg)](https://docs.rs/narrow)

An experimental (work-in-progress) implementation of [Apache Arrow](https://arrow.apache.org).

This crate provides types to support reading and writing instances of abstract data types in Arrow's in-memory data structures.

# Example

```rust
use narrow::{
    array::{StructArray, UnionArray},
    ArrayType, Length,
};

#[derive(ArrayType, Default)]
struct Foo {
    a: bool,
    b: u32,
    c: Option<String>,
}

#[derive(ArrayType, Default)]
struct Bar(Vec<u8>);

#[derive(ArrayType)]
enum FooBar {
    Foo(Foo),
    Bar(Bar),
    None,
}

let struct_array = [
    Foo {
        a: false,
        b: 0,
        c: None,
    },
    Foo {
        a: true,
        b: 42,
        c: Some("hello world".to_owned()),
    },
]
.into_iter()
.collect::<StructArray<Foo>>();
assert_eq!(struct_array.len(), 2);

let union_array = [
    FooBar::Foo(Foo {
        a: true,
        b: 42,
        c: Some("hello world".to_owned()),
    }),
    FooBar::Bar(Bar(vec![1, 2, 3, 4])),
    FooBar::None,
    FooBar::None,
]
.into_iter()
.collect::<UnionArray<FooBar, 3>>();
assert_eq!(union_array.len(), 4);
```

# Features

The crate supports the following optional features:

- `derive`: adds [`ArrayType`] derive support.
- `arrow-rs`: adds array conversion methods for [arrow](https://docs.rs/arrow).
- `uuid`: adds `ArrayType` support for [uuid::Uuid](https://docs.rs/uuid/latest/uuid/struct.Uuid.html).

# Docs

- [Docs (release)](https://docs.rs/narrow)
- [Docs (`main`)](https://mbrobbel.github.io/narrow/)

# Minimum supported Rust version

The minimum supported Rust version for this crate is Rust 1.70.0.

# License

Licensed under either of [Apache License, Version 2.0](https://github.com/mbrobbel/narrow/blob/main/LICENSE-APACHE) or [MIT license](https://github.com/mbrobbel/narrow/blob/main/LICENSE-MIT) at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
