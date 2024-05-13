![Narrow logo](narrow.svg)

# Narrow

[![crates.io](https://img.shields.io/crates/v/narrow.svg)](https://crates.io/crates/narrow)
[![docs.rs](https://docs.rs/narrow/badge.svg)](https://docs.rs/narrow)

An experimental (work-in-progress) statically typed implementation of [Apache Arrow](https://arrow.apache.org).

This crate provides methods to automatically generate types to support reading and writing instances of abstract data types in Arrow's in-memory data structures.

## Why

- The [arrow](https://docs.rs/arrow) crate provides APIs that mostly make sense when array types are only known at runtime, however, for applications with [types known at compile-time](https://docs.rs/arrow/latest/arrow/#type-erasure--trait-objects), using [trait objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html) and [downcasting](https://docs.rs/arrow/latest/arrow/array/fn.downcast_array.html) is not ergonomic.
- Builders for [nested](https://docs.rs/arrow/latest/arrow/datatypes/enum.DataType.html#method.is_nested) array types are [complex](https://docs.rs/arrow/latest/arrow/array/struct.StructBuilder.html) and error-prone.

There are [other crates](https://crates.io/search?q=arrow%20derive&sort=relevance) that aim to prevent users from having to maintain array builder code by providing derive macros. These builders typically produce type-erased arrays, whereas this crate only provides fully statically typed arrays.

### Goals and non-goals

#### Goals

- Provide production ready, fully statically typed, safe and efficient Arrow array implementations
- Enable everyone using Rust to easily benefit from the Arrow ecosystem
- Provide zero-copy interop with the [arrow](https://docs.rs/arrow) crate
- Support custom buffer implementations e.g. to support accelerators
- Explore expressing Arrow concepts using the Rust type system, and mapping Rust concepts to Arrow

#### Non-goals

- Support arbitrary array types at runtime (the [arrow](https://docs.rs/arrow) crate supports this use case)
- Provide compute kernels
- Replace other Arrow implementations

## Docs

- [Docs (release)](https://docs.rs/narrow)
- [Docs (main)](https://mbrobbel.github.io/narrow/)

## Minimum supported Rust version

The minimum supported Rust version for this crate is Rust 1.70.0.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
