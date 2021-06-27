# Narrow

A Rust implementation of [Apache Arrow](https://arrow.apache.org).

## Docs

- [Docs (main)](https://mbrobbel.github.io/narrow/narrow/index.html)

## Progress

- [x] Buffer
- [x] Bitmap
- [x] Nullable
- [x] Validity
- [x] Offset
- [ ] Array
  - [x] Fixed-size primitive
  - [x] Boolean
  - [ ] Variable-size binary
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

## Minimum supported Rust version

The minimum supported Rust version is 1.53.

## Example

```rust
use narrow::{Array, BooleanArray, Float32Array, StructArray, Uint8Array};

#[derive(Array, Copy, Clone, Debug, PartialEq)]
pub struct Person {
    age: u8,
    happy: bool,
    distance: Option<f32>,
}

let persons = vec![
    Person {
        age: 20,
        happy: true,
        distance: Some(1.5),
    },
    Person {
        age: 22,
        happy: true,
        distance: None,
    },
];

// Convert array of structs to struct of arrays.
let persons_array: StructArray<Person, false> = persons.clone().into_iter().collect();
assert_eq!(persons_array.len(), 2);

// To sum the ages we only need to read the `age` array.
let age_array: &Uint8Array<false> = &persons_array.age;
assert_eq!(age_array.into_iter().sum::<u8>(), 42);

// Make sure everyone is happy.
let happy_array: &BooleanArray<false> = &persons_array.happy;
assert!(happy_array.into_iter().all(|bool| bool));

// The const generic bool in array types indicates nullability of the
// values in that array.
let distance_array: &Float32Array<true> = &persons_array.distance;
assert_eq!(distance_array.null_count(), 1);

// Convert struct of arrays back to array of structs
assert_eq!(persons_array.into_iter().collect::<Vec<_>>(), persons);
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
