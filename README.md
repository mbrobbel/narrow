# Narrow

A Rust implementation of [Apache Arrow](https://arrow.apache.org).

## Progress

- [x] Buffer
- [x] Bitmap
- [x] Nullable
- [x] Validity
- [x] Offset
- [ ] Array
  - [x] Fixed-size primitive
  - [x] Boolean
  - [x] Variable-size binary
  - [ ] Variable-size list
  - [ ] Fixed-size list
  - [ ] Struct
  - [ ] Union
  - [ ] Null
  - [ ] Dictionary
- [ ] Logical types
- [ ] Schema
- [ ] RecordBatch
- [ ] DictionaryBatch
- [ ] Table
- [ ] IPC
  - [ ] Streaming
  - [ ] File
- [ ] Flight
- [ ] Documentation
- [ ] Benchmarks

## Docs

- [Docs (main)](https://mbrobbel.github.io/narrow/narrow/index.html)

## Minimum supported Rust version

The minimum supported Rust version is 1.51 due to the use of [const generics](https://rust-lang.github.io/rfcs/2000-const-generics.html).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
