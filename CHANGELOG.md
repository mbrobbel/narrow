

## v0.5.0 (2024-02-06)

### New Features (BREAKING)

 - <csr-id-a2276a006e5b08348cbc57b5870968b90738caa8/> add parquet write support for `UnionArray`

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#149](https://github.com/mbrobbel/narrow/issues/149)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#149](https://github.com/mbrobbel/narrow/issues/149)**
    - Add parquet write support for `UnionArray` ([`a2276a0`](https://github.com/mbrobbel/narrow/commit/a2276a006e5b08348cbc57b5870968b90738caa8))
</details>

## v0.4.4 (2024-02-06)

### New Features

 - <csr-id-02c54a7dc49da3e1adcabdad63b22fca2e8fa2c9/> add `arrow` `NullArray` interop

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#147](https://github.com/mbrobbel/narrow/issues/147)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#147](https://github.com/mbrobbel/narrow/issues/147)**
    - Add `arrow` `NullArray` interop ([`02c54a7`](https://github.com/mbrobbel/narrow/commit/02c54a7dc49da3e1adcabdad63b22fca2e8fa2c9))
</details>

## v0.4.3 (2024-02-05)

### New Features

 - <csr-id-645514bed37dc9329a1673a60cff353664a90ef3/> add `UnionArray`

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 5 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#146](https://github.com/mbrobbel/narrow/issues/146)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#146](https://github.com/mbrobbel/narrow/issues/146)**
    - Add `UnionArray` ([`645514b`](https://github.com/mbrobbel/narrow/commit/645514bed37dc9329a1673a60cff353664a90ef3))
</details>

## v0.4.2 (2024-01-31)

### Documentation

 - <csr-id-d0276bcb145826120d731ec9fcac2771025a499d/> update `UuidArray`'s documentation

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 day passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#144](https://github.com/mbrobbel/narrow/issues/144)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#144](https://github.com/mbrobbel/narrow/issues/144)**
    - Update `UuidArray`'s documentation ([`d0276bc`](https://github.com/mbrobbel/narrow/commit/d0276bcb145826120d731ec9fcac2771025a499d))
</details>

## v0.4.1 (2024-01-29)

### New Features

 - <csr-id-e576ed3a4964a50a850610b881d455319b76fe76/> `LogicalArray` to support logical types

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 7 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#124](https://github.com/mbrobbel/narrow/issues/124)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#124](https://github.com/mbrobbel/narrow/issues/124)**
    - `LogicalArray` to support logical types ([`e576ed3`](https://github.com/mbrobbel/narrow/commit/e576ed3a4964a50a850610b881d455319b76fe76))
</details>

## v0.4.0 (2024-01-22)

<csr-id-117a4f383870446e39ad5cb3593e56f6dda09ca1/>
<csr-id-11c8970d7e3335914f7eb511e3b115ff2edd0de2/>

### Chore

 - <csr-id-117a4f383870446e39ad5cb3593e56f6dda09ca1/> fix clippy 1.75.0 warnings
   Fixes new 1.75.0 clippy warnings.
 - <csr-id-11c8970d7e3335914f7eb511e3b115ff2edd0de2/> enable more lints

### New Features

 - <csr-id-fcb49b34bcadbdd6e1e51534ae1b94a6b896c8c6/> convert `StructArray` from `arrow_array::StructArray`
   This enables roundtripping through a parquet file.
 - <csr-id-b4c49b09601a23df564157f9df6de12ac692142b/> convert `StructArray` from `arrow_array::StructArray`
 - <csr-id-b5210c7a558d4c665a93f609d26d5882f17a3970/> add `arrow-rs` interop support for `FixedSizeListArray`
   Adds support for interop between `narrow::array::FixedSizeListArray` and
   `arrow_array::FixedSizeListArray`.
 - <csr-id-b4d403802f3321875762486f2fb90d34b424fe56/> add `arrow-rs` interop support for `FixedSizeListArray`
 - <csr-id-e84f00ce5c12c27bee0d53cf46c94b86af55f184/> add `FixedSizeListArray`
 - <csr-id-bfc13993867f3e2bb496cd28e850c2cca64746e2/> add non-nullable to nullable conversion

### Bug Fixes

 - <csr-id-fabc404518bc639c0d84ca499b112b70fc4362b5/> bound on `ArrayType` implementation for arrays
   The `FixedSize` bound was used when arrays were stored in
   `FixedSizePrimitiveArray`.
 - <csr-id-54eda3c6d9938a28103efb379324292cda5f389f/> clippy warning
 - <csr-id-59cdb4af4b72656d21c9bfeac9d61c1c3dfe0e0d/> remove comment
 - <csr-id-b6b922c7b9948fcab9491d0ca739879863f6473c/> remove `FixedSize` impl for tuple
 - <csr-id-10aacdf0ee143770f9f0c09134e6b7e865358fd8/> offset extend impl for nullable data
   For nullable data it should also flatten the option iterator.
 - <csr-id-60fb809c72704657cd5bf850f3f43c2f468beb7a/> offset extend impl for nullable data

### New Features (BREAKING)

 - <csr-id-374aedf4a7e5b875516f11fb03544d3470d4ae19/> add `arrow-rs` features for buffer and array interop
   Adds interop with `arrow-rs`. The added `parquet` example demonstrates
   what this enables.
 - <csr-id-36b2343fb7b95d38e71147031b700c97e273df18/> add `FixedSizeListArray`
 - <csr-id-3b60bbe4dadd67917e07ee22f2cadc91be47e0fa/> add `OffsetElement` and `UnionType` to the `Array` GAT of `ArrayType`
   This adds generics for offset element type (`i32` or `i64`) and union
   layout (sparse or dense) to the `Array` type constructor of the
   `ArrayType` trait.
   
   This is not ideal without default types for the generics in a generic
   associated type, but the alternatives are worse (making `ArrayType`
   generic over these types with defaults).
 - <csr-id-a3613c534c43ff51e4a163a20d90e6d24168d6a3/> add `OffsetElement` and `UnionType` to the `Array` GAT of `ArrayType`
 - <csr-id-7db53c26d3b6a9666f1dc1a91ee298384c273a02/> add item associated type to `Unit` trait
   To support using `NullArray` for unit variants of enums in
   `UnionArrays`, this adds an `Item` associated type to the `Unit` trait,
   which converts into the type implementing `Unit`, allowing code
   generation of types for unit enum variants which implement `Unit` and
   convert to instances of the variants of the original enum.
 - <csr-id-3a7f327f6d2ecce592a3f0abeb6d2ce9fdb57aed/> add item associated type to `Unit` trait

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 52 commits contributed to the release over the course of 173 calendar days.
 - 173 days passed between releases.
 - 20 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 11 unique issues were worked on: [#100](https://github.com/mbrobbel/narrow/issues/100), [#102](https://github.com/mbrobbel/narrow/issues/102), [#107](https://github.com/mbrobbel/narrow/issues/107), [#108](https://github.com/mbrobbel/narrow/issues/108), [#109](https://github.com/mbrobbel/narrow/issues/109), [#110](https://github.com/mbrobbel/narrow/issues/110), [#117](https://github.com/mbrobbel/narrow/issues/117), [#118](https://github.com/mbrobbel/narrow/issues/118), [#123](https://github.com/mbrobbel/narrow/issues/123), [#136](https://github.com/mbrobbel/narrow/issues/136), [#98](https://github.com/mbrobbel/narrow/issues/98)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#100](https://github.com/mbrobbel/narrow/issues/100)**
    - Add `arrow-rs` features for buffer and array interop ([`374aedf`](https://github.com/mbrobbel/narrow/commit/374aedf4a7e5b875516f11fb03544d3470d4ae19))
 * **[#102](https://github.com/mbrobbel/narrow/issues/102)**
    - Add non-nullable to nullable conversion ([`bfc1399`](https://github.com/mbrobbel/narrow/commit/bfc13993867f3e2bb496cd28e850c2cca64746e2))
 * **[#107](https://github.com/mbrobbel/narrow/issues/107)**
    - Add `Index` trait ([`fa91089`](https://github.com/mbrobbel/narrow/commit/fa91089d342713d6bf9b678811184cdc8a9962a1))
 * **[#108](https://github.com/mbrobbel/narrow/issues/108)**
    - Add item associated type to `Unit` trait ([`7db53c2`](https://github.com/mbrobbel/narrow/commit/7db53c26d3b6a9666f1dc1a91ee298384c273a02))
 * **[#109](https://github.com/mbrobbel/narrow/issues/109)**
    - Add `OffsetElement` and `UnionType` to the `Array` GAT of `ArrayType` ([`3b60bbe`](https://github.com/mbrobbel/narrow/commit/3b60bbe4dadd67917e07ee22f2cadc91be47e0fa))
 * **[#110](https://github.com/mbrobbel/narrow/issues/110)**
    - Add `FixedSizeListArray` ([`36b2343`](https://github.com/mbrobbel/narrow/commit/36b2343fb7b95d38e71147031b700c97e273df18))
 * **[#117](https://github.com/mbrobbel/narrow/issues/117)**
    - Add `arrow-rs` interop support for `FixedSizeListArray` ([`b5210c7`](https://github.com/mbrobbel/narrow/commit/b5210c7a558d4c665a93f609d26d5882f17a3970))
 * **[#118](https://github.com/mbrobbel/narrow/issues/118)**
    - Convert `StructArray` from `arrow_array::StructArray` ([`fcb49b3`](https://github.com/mbrobbel/narrow/commit/fcb49b34bcadbdd6e1e51534ae1b94a6b896c8c6))
 * **[#123](https://github.com/mbrobbel/narrow/issues/123)**
    - Bound on `ArrayType` implementation for arrays ([`fabc404`](https://github.com/mbrobbel/narrow/commit/fabc404518bc639c0d84ca499b112b70fc4362b5))
 * **[#136](https://github.com/mbrobbel/narrow/issues/136)**
    - Fix clippy 1.75.0 warnings ([`117a4f3`](https://github.com/mbrobbel/narrow/commit/117a4f383870446e39ad5cb3593e56f6dda09ca1))
 * **[#98](https://github.com/mbrobbel/narrow/issues/98)**
    - Offset extend impl for nullable data ([`10aacdf`](https://github.com/mbrobbel/narrow/commit/10aacdf0ee143770f9f0c09134e6b7e865358fd8))
 * **Uncategorized**
    - Convert `StructArray` from `arrow_array::StructArray` ([`b4c49b0`](https://github.com/mbrobbel/narrow/commit/b4c49b09601a23df564157f9df6de12ac692142b))
    - Add missing tests ([`6c703bd`](https://github.com/mbrobbel/narrow/commit/6c703bd829f73ebffdb0bba17e22c9b6d145d112))
    - Clippy warning ([`54eda3c`](https://github.com/mbrobbel/narrow/commit/54eda3c6d9938a28103efb379324292cda5f389f))
    - Add `arrow-rs` interop support for `FixedSizeListArray` ([`b4d4038`](https://github.com/mbrobbel/narrow/commit/b4d403802f3321875762486f2fb90d34b424fe56))
    - Add direct `RecordBatch` conversion for `StructArray` ([`8f0c5b2`](https://github.com/mbrobbel/narrow/commit/8f0c5b23ddc769dddb7e3c7b762a06d4528603fa))
    - Remove a comment ([`d9c076f`](https://github.com/mbrobbel/narrow/commit/d9c076f727fd2ba3554002cf12d1785654b4f2a8))
    - Add parquet example ([`094f3a0`](https://github.com/mbrobbel/narrow/commit/094f3a0cc51916d3c8d7bcec11b778a6ed46769c))
    - Generalize more string array methods ([`bc0f459`](https://github.com/mbrobbel/narrow/commit/bc0f4595a6177ff8a7869bbc77d53cec3a839a12))
    - Generalize stringarray extend impl ([`66b4970`](https://github.com/mbrobbel/narrow/commit/66b497031f655497b203b06bb8fcf30439a8f71a))
    - Split out implementations and add more tests ([`ea99db7`](https://github.com/mbrobbel/narrow/commit/ea99db71a558f3397a5aa169f95860bfae92d0dd))
    - Fix some clippy warnings ([`25c7367`](https://github.com/mbrobbel/narrow/commit/25c7367b5eb2ada1a2ae5b3fad4c695e9956fa3e))
    - Change interaction with `ArrowNativeType` ([`6c43438`](https://github.com/mbrobbel/narrow/commit/6c4343880fc55c9191fee30c96a3ecf5c513622e))
    - Merge branch 'main' into arrow-array ([`8ee0a2c`](https://github.com/mbrobbel/narrow/commit/8ee0a2cdf0ccac0299dae175c79a3730e440a55f))
    - Add nested test ([`63c6e8d`](https://github.com/mbrobbel/narrow/commit/63c6e8df8df779940fab20a1c418e275732ba5d2))
    - Add `IntoIterator` implementation, change `ArrayType` for `[T: FixedSize; N]` ([`b631643`](https://github.com/mbrobbel/narrow/commit/b6316436287a64547e32942eef117fbba8283b14))
    - Add `FixedSizeListArray` ([`e84f00c`](https://github.com/mbrobbel/narrow/commit/e84f00ce5c12c27bee0d53cf46c94b86af55f184))
    - Add `OffsetElement` and `UnionType` to the `Array` GAT of `ArrayType` ([`a3613c5`](https://github.com/mbrobbel/narrow/commit/a3613c534c43ff51e4a163a20d90e6d24168d6a3))
    - Add item associated type to `Unit` trait ([`3a7f327`](https://github.com/mbrobbel/narrow/commit/3a7f327f6d2ecce592a3f0abeb6d2ce9fdb57aed))
    - Fix docs ([`d440189`](https://github.com/mbrobbel/narrow/commit/d440189b39aae772c8e85c691fd3c5fd84897529))
    - Use `ArrayBuffer<1>` for `SingleBuffer` ([`34db73e`](https://github.com/mbrobbel/narrow/commit/34db73e1bdf43ae3d65f53278edbdb2386586704))
    - Add indexing to `Offset` ([`a84ee37`](https://github.com/mbrobbel/narrow/commit/a84ee37861584d9e4d8d54d29c1826180f597c26))
    - Add `Index` trait ([`97747b5`](https://github.com/mbrobbel/narrow/commit/97747b5d2620d1ce14e26b0c2b3e01dcd8d27b91))
    - Remove comment ([`59cdb4a`](https://github.com/mbrobbel/narrow/commit/59cdb4af4b72656d21c9bfeac9d61c1c3dfe0e0d))
    - Enable more lints ([`11c8970`](https://github.com/mbrobbel/narrow/commit/11c8970d7e3335914f7eb511e3b115ff2edd0de2))
    - Merge branch 'main' into arrow-array ([`8ddf760`](https://github.com/mbrobbel/narrow/commit/8ddf7602a212a597ab2b59f37873cb8dc8c7d214))
    - Add non-nullable to nullable conversion ([`11f75e0`](https://github.com/mbrobbel/narrow/commit/11f75e05af187c7f17bf63b44c7ae03e4c78cf3d))
    - Add `BooleanArray` conversion ([`21547b1`](https://github.com/mbrobbel/narrow/commit/21547b16b5823c67c81471c9cdc4cfda0137963d))
    - Rename feature to `arrow-rs` ([`afbb962`](https://github.com/mbrobbel/narrow/commit/afbb9624ba0307c60f0e8ec828e2efe5c37967ea))
    - Move `arrow-rs` interop to `arrow` module ([`e39ad3b`](https://github.com/mbrobbel/narrow/commit/e39ad3b169c620524a31a8d652203065e2bddfad))
    - Fix warning ([`c5c09f0`](https://github.com/mbrobbel/narrow/commit/c5c09f0b311f054a3f604664ac3539db70f40351))
    - Merge branch 'main' into arrow-array ([`42ba214`](https://github.com/mbrobbel/narrow/commit/42ba2140040bde130feb58278fc0b0a9ca6465f8))
    - Remove `FixedSize` impl for tuple ([`b6b922c`](https://github.com/mbrobbel/narrow/commit/b6b922c7b9948fcab9491d0ca739879863f6473c))
    - Implicit conversion to `ArrowBuffer` from `VecBuffer` is now supported ([`9494ce2`](https://github.com/mbrobbel/narrow/commit/9494ce2f8bf9058e9981f1c1bda0561f93e4f317))
    - Merge branch 'main' into arrow-array ([`bdb17bc`](https://github.com/mbrobbel/narrow/commit/bdb17bc66b570dc0bada75273604425e046b156c))
    - Add `BufferType` implementation for `arrow_buffer::ScalarBuffer` ([`44e3567`](https://github.com/mbrobbel/narrow/commit/44e35671a0935ad82c10b93960c8667af09d5dc8))
    - Make conversion generic over buffer type ([`e2b40f1`](https://github.com/mbrobbel/narrow/commit/e2b40f1c10c41132a764d3a67342d0afa9abf44f))
    - Add `arrow-array` feature for zero-copy array interop ([`26e746c`](https://github.com/mbrobbel/narrow/commit/26e746cb40da0a90cb8311a3e4108256f57859b9))
    - Use `BufferBuilder` abstraction ([`7b5dab9`](https://github.com/mbrobbel/narrow/commit/7b5dab923acb8ed8656bc6567a6a0615bbedcf43))
    - Offset extend impl for nullable data ([`60fb809`](https://github.com/mbrobbel/narrow/commit/60fb809c72704657cd5bf850f3f43c2f468beb7a))
    - Some fixes and tests ([`3cfe877`](https://github.com/mbrobbel/narrow/commit/3cfe877c2db7c29a9ec91f02e34102f12c1e1588))
    - Setup `arrow-buffer` interop ([`4d0d333`](https://github.com/mbrobbel/narrow/commit/4d0d33390521c369f26f9aa2940f13c6266d0ad7))
</details>

## v0.3.4 (2023-08-01)

### Bug Fixes

 - <csr-id-8fb5f2f5b2559a5c77efc7193514befad815cddb/> `ArrayType` derive for named structs
   ```rust
   #[derive(ArrayType, Default)]
   struct Bar<T> {
   a: u32,
   b: Option<bool>,
   c: T,
   }
   
   let input = [
   Some(Bar {
   a: 1,
   b: Some(false),
   c: None,
   }),
   None,
   Some(Bar {
   a: 2,
   b: None,
   c: Some(()),
   }),
   ];
   
   let array = input.into_iter().collect::<StructArray<Bar<_>, true>>();
   assert_eq!(array.len(), 3);
   assert_eq!(array.is_valid(0), Some(true));
   assert_eq!(array.is_null(1), Some(true));
   assert_eq!(array.is_valid(2), Some(true));
   
   let int_array = &array.0.as_ref().a;
   assert_eq!(int_array.0.as_slice(), &[1, Default::default(), 2]);
   
   let bool_array = &array.0.as_ref().b;
   assert_eq!(
   bool_array.into_iter().collect::<Vec<_>>(),
   &[Some(false), None, None]
   );
   
   let null_array = &array.0.as_ref().c;
   assert_eq!(null_array.is_null(0), Some(true));
   assert_eq!(null_array.is_null(1), Some(true));
   assert_eq!(null_array.is_valid(2), Some(true));
   
   let input = [
   Some(Bar {
   a: 1,
   b: None,
   c: false,
   }),
   None,
   ];
   let array = input.into_iter().collect::<StructArray<Bar<_>, true>>();
   assert_eq!(array.len(), 2);
   ```

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 4 calendar days.
 - 4 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#82](https://github.com/mbrobbel/narrow/issues/82)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#82](https://github.com/mbrobbel/narrow/issues/82)**
    - `ArrayType` derive for named structs ([`8fb5f2f`](https://github.com/mbrobbel/narrow/commit/8fb5f2f5b2559a5c77efc7193514befad815cddb))
 * **Uncategorized**
    - Consolidate the common items for the different field types ([`427bebb`](https://github.com/mbrobbel/narrow/commit/427bebbd80dadead8260440b02ec283e29cc58ef))
    - Add derive support for named structs ([`538c16c`](https://github.com/mbrobbel/narrow/commit/538c16c1e47673b0a924e3fd9b3208e00ead36ec))
</details>

## v0.3.3 (2023-07-27)

### Bug Fixes

 - <csr-id-9a48422f4a8de0f9b5d109ce44c4c9a14544116a/> `ArrayType` derive for tuple structs
   ```rust
   #[derive(ArrayType, Default)]
   struct Foo<'a>(u32, u16, &'a str);
   
   #[derive(ArrayType, Default)]
   struct Bar<'a>(Foo<'a>);
   
   #[derive(ArrayType, Default)]
   struct FooBar<'a, T>(Bar<'a>, T);
   
   let input = [
   FooBar(Bar(Foo(1, 2, "n")), false),
   FooBar(Bar(Foo(1, 2, "arrow")), false),
   ];
   let array = input.into_iter().collect::<StructArray<FooBar<_>>>();
   assert_eq!(array.len(), 2);
   
   let input = vec![
   Some(vec![Some(FooBar(Bar(Foo(42, 0, "!"), 1234))]),
   None,
   Some(vec![None]),
   Some(vec![None, None]),
   ];
   let array = input
   .into_iter()
   .collect::<VariableSizeListArray<StructArray<FooBar<_>, true>, true>>();
   assert_eq!(array.len(), 4);
   ```
 - <csr-id-1db19ad5f65ec2d690e2fbcb1292812bfaba2abb/> `ArrayType` derive for tuple structs

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#80](https://github.com/mbrobbel/narrow/issues/80)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#80](https://github.com/mbrobbel/narrow/issues/80)**
    - `ArrayType` derive for tuple structs ([`9a48422`](https://github.com/mbrobbel/narrow/commit/9a48422f4a8de0f9b5d109ce44c4c9a14544116a))
 * **Uncategorized**
    - `ArrayType` derive for tuple structs ([`1db19ad`](https://github.com/mbrobbel/narrow/commit/1db19ad5f65ec2d690e2fbcb1292812bfaba2abb))
</details>

## v0.3.2 (2023-07-27)

### Bug Fixes

 - <csr-id-a7a3f79a98fc15879aabf677b17e12bb285ce57f/> `ArrayType` derive for unit structs
   Add support to derive `ArrayType` for unit structs:
   ```rust
   #[derive(ArrayType, Copy, Clone, Default)]
   struct Foo;
   
   let array = [Foo; 5].into_iter().collect::<StructArray<Foo>>();
   assert_eq!(array.len(), 5);
   
   let array = [Some(Foo); 5].into_iter().collect::<StructArray<Foo, true>>();
   assert_eq!(array.len(), 5);
   assert!(array.all_valid());
   ```
 - <csr-id-e951ed1510214d09794168f1b385289359b76b1c/> `ArrayType` derive for unit structs

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#79](https://github.com/mbrobbel/narrow/issues/79)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#79](https://github.com/mbrobbel/narrow/issues/79)**
    - `ArrayType` derive for unit structs ([`a7a3f79`](https://github.com/mbrobbel/narrow/commit/a7a3f79a98fc15879aabf677b17e12bb285ce57f))
 * **Uncategorized**
    - Fix test ([`d67a5c9`](https://github.com/mbrobbel/narrow/commit/d67a5c9e6424fcf1865fdc7a13ad0cc6299c4740))
    - `ArrayType` derive for unit structs ([`e951ed1`](https://github.com/mbrobbel/narrow/commit/e951ed1510214d09794168f1b385289359b76b1c))
</details>

## v0.3.1 (2023-07-27)

<csr-id-17bf9944762a9b036fd6d1a5fa5280f2e68dba03/>

### Chore

 - <csr-id-17bf9944762a9b036fd6d1a5fa5280f2e68dba03/> fix gh release

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Fix gh release ([`17bf994`](https://github.com/mbrobbel/narrow/commit/17bf9944762a9b036fd6d1a5fa5280f2e68dba03))
</details>

## v0.3.0 (2023-07-27)

### Documentation

 - <csr-id-10e3027af7051ffcbe3a9d128ac4667881a05b17/> update `Validity` docs
 - <csr-id-12f9079fdf85ab741f02498a1baa820f57e494af/> update `Validity` docs

### Bug Fixes

 - <csr-id-50c9d81cdce63ca6c3500d2c436ce4c3712a506d/> add `BufferRef` and `BufferRefMut` impls for `BooleanArray`

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#77](https://github.com/mbrobbel/narrow/issues/77)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#77](https://github.com/mbrobbel/narrow/issues/77)**
    - Update `Validity` docs ([`10e3027`](https://github.com/mbrobbel/narrow/commit/10e3027af7051ffcbe3a9d128ac4667881a05b17))
 * **Uncategorized**
    - Update `Validity` docs ([`12f9079`](https://github.com/mbrobbel/narrow/commit/12f9079fdf85ab741f02498a1baa820f57e494af))
    - Add `BufferRef` and `BufferRefMut` impls for `BooleanArray` ([`50c9d81`](https://github.com/mbrobbel/narrow/commit/50c9d81cdce63ca6c3500d2c436ce4c3712a506d))
</details>

## v0.2.5 (2023-07-26)

<csr-id-adc3f3b5ff5854ece947fe4dbee33e8d8cf5fff6/>
<csr-id-aeefe446e6845ab203e72685d37fc263e1cbd2a5/>

### Bug Fixes

 - <csr-id-ce9a69ec685371790fc4acc4713d1390470a4289/> rename buffer generic of `Nullable` to match other generic buffers
 - <csr-id-1e2265e2d12cd07121dde541f611cb8c350400a3/> add missing `BitmapRef`, `BitmapRefMut` and `ValidityBitmap` implementations
   When arrays are nullable they should provide access to the validity
   bitmap and the methods of the `ValidityBitmap` trait to get nullability
   information.
 - <csr-id-faeca97ab9785b3d9f2c55ac9ab94ba90a9c1c6f/> add missing `BitmapRef`, `BitmapRefMut` and `ValidityBitmap` implementations

### Test

 - <csr-id-adc3f3b5ff5854ece947fe4dbee33e8d8cf5fff6/> `FromIterator` for `VariableSizeListArray` with nullable child array
 - <csr-id-aeefe446e6845ab203e72685d37fc263e1cbd2a5/> `FromIterator` for nested nullable `VariableSizeListArray`

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#74](https://github.com/mbrobbel/narrow/issues/74)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#74](https://github.com/mbrobbel/narrow/issues/74)**
    - Add missing `BitmapRef`, `BitmapRefMut` and `ValidityBitmap` implementations ([`1e2265e`](https://github.com/mbrobbel/narrow/commit/1e2265e2d12cd07121dde541f611cb8c350400a3))
 * **Uncategorized**
    - Rename buffer generic of `Nullable` to match other generic buffers ([`ce9a69e`](https://github.com/mbrobbel/narrow/commit/ce9a69ec685371790fc4acc4713d1390470a4289))
    - `FromIterator` for `VariableSizeListArray` with nullable child array ([`adc3f3b`](https://github.com/mbrobbel/narrow/commit/adc3f3b5ff5854ece947fe4dbee33e8d8cf5fff6))
    - `FromIterator` for nested nullable `VariableSizeListArray` ([`aeefe44`](https://github.com/mbrobbel/narrow/commit/aeefe446e6845ab203e72685d37fc263e1cbd2a5))
    - Add missing `BitmapRef`, `BitmapRefMut` and `ValidityBitmap` implementations ([`faeca97`](https://github.com/mbrobbel/narrow/commit/faeca97ab9785b3d9f2c55ac9ab94ba90a9c1c6f))
</details>

## v0.2.2 (2023-07-26)

<csr-id-3cbea45adf8eb3095220f8e55f78327eb9798036/>

### Chore

 - <csr-id-3cbea45adf8eb3095220f8e55f78327eb9798036/> remove some comments

### Bug Fixes

 - <csr-id-f244d3afb4e6daad54afe796f0d84f053b2f1b26/> specify derive crate version to fix publish

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 62 commits contributed to the release over the course of 630 calendar days.
 - 726 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#69](https://github.com/mbrobbel/narrow/issues/69), [#70](https://github.com/mbrobbel/narrow/issues/70)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#69](https://github.com/mbrobbel/narrow/issues/69)**
    - Remove some comments ([`3cbea45`](https://github.com/mbrobbel/narrow/commit/3cbea45adf8eb3095220f8e55f78327eb9798036))
 * **[#70](https://github.com/mbrobbel/narrow/issues/70)**
    - Specify derive crate version to fix publish ([`f244d3a`](https://github.com/mbrobbel/narrow/commit/f244d3afb4e6daad54afe796f0d84f053b2f1b26))
 * **Uncategorized**
    - Specify derive crate version to fix publish ([`e192fd5`](https://github.com/mbrobbel/narrow/commit/e192fd541649954faa7b69b2c2dc1a0939a20133))
    - Remove some comments ([`048f330`](https://github.com/mbrobbel/narrow/commit/048f330edc5df2ddc370cf81d8a59fe0b909ad51))
    - Merge #62 ([`5deccd3`](https://github.com/mbrobbel/narrow/commit/5deccd3dfd6e1e7d05566c8db5fe6aff52ba6072))
    - Fix clippy warning ([`9b7b2e6`](https://github.com/mbrobbel/narrow/commit/9b7b2e6b459bf6fddc03ff2d808476c5d27b0810))
    - Clean up ([`b5a048f`](https://github.com/mbrobbel/narrow/commit/b5a048fd58f097ce3d0a35f2410a1b5da54aec71))
    - Update arrays with different offset implementation. Data no longer gets default values for nulls. ([`c646637`](https://github.com/mbrobbel/narrow/commit/c646637a4aed03aab49323aa80cee6d15a1b9963))
    - A different offset abstraction ([`195c1e9`](https://github.com/mbrobbel/narrow/commit/195c1e92b307dd89075d3db044cf52163ba38738))
    - Add variable size list array ([`299a9c3`](https://github.com/mbrobbel/narrow/commit/299a9c3d01ec7e3db4df62704edc7ec3597dbc2b))
    - Add more array types ([`9d5a01f`](https://github.com/mbrobbel/narrow/commit/9d5a01f55d3d6aa574d4fd715a98fbc0e5c8f5d4))
    - Add `BooleanArray` ([`f121d15`](https://github.com/mbrobbel/narrow/commit/f121d154dd2687faf02c54ca7f4f4a6b9ad7e968))
    - Drop `FixedSize` impl for tuples with more than 1 field ([`1fdf1ab`](https://github.com/mbrobbel/narrow/commit/1fdf1abb96edd24e6e8921651552a5bfecb52d0e))
    - Change ~some~ all things ([`952ccf0`](https://github.com/mbrobbel/narrow/commit/952ccf0cbc69d18a3624dc3699a57905bfeb18be))
    - Merge #60 ([`35b199c`](https://github.com/mbrobbel/narrow/commit/35b199cc35233f93829404ed6cb85aacf987f324))
    - Fix clippy warning ([`d76f69d`](https://github.com/mbrobbel/narrow/commit/d76f69dc9263a77f4919f820ea13afb9c4cfd852))
    - Merge #54 ([`65f518a`](https://github.com/mbrobbel/narrow/commit/65f518aad8060a5a033ab05f7fef9ebb13adffce))
    - Add `ValidityBitmap::validity_bitmap_mut` and improve test ([`aea75f2`](https://github.com/mbrobbel/narrow/commit/aea75f2f2371cd52b9d10d2764a4107484faa66b))
    - Add some flexibility to `Nullable` ([`4949c03`](https://github.com/mbrobbel/narrow/commit/4949c03e95f5de7278f652759b6944439c84593e))
    - Merge #53 ([`411b13d`](https://github.com/mbrobbel/narrow/commit/411b13ddfdfe311e97dd8cf0c3e66f6bef22ab8f))
    - Implement `ArrayType` for `str` ([`6824595`](https://github.com/mbrobbel/narrow/commit/6824595ed7feb2291b8a27cd178f5eb552886ff3))
    - Merge #50 ([`8edbfdc`](https://github.com/mbrobbel/narrow/commit/8edbfdce61be8d1365f5cab2f9b33fb79335a2db))
    - Add RunEndEncoded array ([`804f552`](https://github.com/mbrobbel/narrow/commit/804f55216f23589382d173bbdadb581b32ecd15b))
    - Merge #49 ([`7ba4008`](https://github.com/mbrobbel/narrow/commit/7ba4008fd164be59a58bee16cd8fa3ed9411c4d3))
    - Address review comments ([`f8d4f64`](https://github.com/mbrobbel/narrow/commit/f8d4f645f3b49357e41785df5319f8e40e41f17e))
    - Cargo clippy ([`fe182c7`](https://github.com/mbrobbel/narrow/commit/fe182c7d0f084b54a4c0f38ccb1dce39ae86a9da))
    - Impl ValidityBitmap from StringArray ([`19ae85c`](https://github.com/mbrobbel/narrow/commit/19ae85c5b6b42d4c7a2cbd6abca31c57676e40c3))
    - Merge #47 ([`f4e88e6`](https://github.com/mbrobbel/narrow/commit/f4e88e65cee6e01dcd03485789465d2af2f31244))
    - Fix formatting ([`18cf56a`](https://github.com/mbrobbel/narrow/commit/18cf56af4209a02564e2f53e705cbfd6c9313c4e))
    - Fix more clippy warnings ([`958d948`](https://github.com/mbrobbel/narrow/commit/958d94872074014318715ba1d06f909505e7f344))
    - Fix clippy warnings ([`0d5ebd4`](https://github.com/mbrobbel/narrow/commit/0d5ebd40bb7864b47e9007750affce9aab6be192))
    - Merge #44 ([`747d40c`](https://github.com/mbrobbel/narrow/commit/747d40c985c4de99a99e52dcf3f20350bf2cee72))
    - Add logo to rustdoc ([`cc0735e`](https://github.com/mbrobbel/narrow/commit/cc0735e6d558e3ea7004b15bb3144dc575be1955))
    - Merge #40 ([`69d6758`](https://github.com/mbrobbel/narrow/commit/69d6758f40cf521f0964741a47e9eb725f40c5cf))
    - Add bitmap buffer trait bound ([`a2b1ac1`](https://github.com/mbrobbel/narrow/commit/a2b1ac1d134bb8918a6a34df8d532175c4a31736))
    - Compile using stable Rust ([`74cdd1f`](https://github.com/mbrobbel/narrow/commit/74cdd1f77443a33456e832b2ab47f07edf5945d4))
    - Merge #37 ([`80ae645`](https://github.com/mbrobbel/narrow/commit/80ae6452950512a5860cbb2d154f7d4883573dca))
    - The feature `generic_associated_types` has been stable since 1.66.0-nightly ([`65206cb`](https://github.com/mbrobbel/narrow/commit/65206cb98201110563a2570920b2fb1abefd57b1))
    - Merge #36 ([`e43cb5b`](https://github.com/mbrobbel/narrow/commit/e43cb5b428f7d5c96492aecf5a00ed388f438ae8))
    - Fix clippy warnings ([`280e4b8`](https://github.com/mbrobbel/narrow/commit/280e4b894dd596ddfabd426cfeaf1b5a420e222a))
    - Merge #35 ([`34c4399`](https://github.com/mbrobbel/narrow/commit/34c4399aaeb3edc23ca84eee494c1cd3f3d28131))
    - Add Nullable and Validity ([`9a29600`](https://github.com/mbrobbel/narrow/commit/9a2960045d13051fd31949ac1df2c0a7b88fb74f))
    - Merge #34 ([`9582618`](https://github.com/mbrobbel/narrow/commit/95826183b8ac32dfb41a864b53637388a0399a34))
    - Add offset to Bitmap for zero-copy slicing on non-byte boundaries ([`b663f01`](https://github.com/mbrobbel/narrow/commit/b663f01cd960c78219da2066e2abdd02223d0817))
    - Merge #28 ([`51ac77b`](https://github.com/mbrobbel/narrow/commit/51ac77b24414046aaa095b75090d8216ca520f69))
    - Remove stuff from previous iteration ([`2eab5f1`](https://github.com/mbrobbel/narrow/commit/2eab5f1dbe630faef3af5d587bb54612dc2da1d3))
    - Remove Primitive impl for usize and isize ([`17316be`](https://github.com/mbrobbel/narrow/commit/17316be2e18269bb2faeffd8f7397bfb812b80f2))
    - Fix unresolved doc link to Offset ([`4b81186`](https://github.com/mbrobbel/narrow/commit/4b81186840d3719aede603635345d61b4fc7f5af))
    - Starting afresh ([`aa7f5c2`](https://github.com/mbrobbel/narrow/commit/aa7f5c2881006a8c79c9716f62c0dfeb9387405a))
    - Merge #26 ([`474f797`](https://github.com/mbrobbel/narrow/commit/474f7975231db518f0bdce024353b47ca9410dd5))
    - Fix ArrayType impl for Rust array types ([`c170c3d`](https://github.com/mbrobbel/narrow/commit/c170c3d36f121593c032f5d5bcb431d1de8ea716))
    - Merge #25 ([`15a77e7`](https://github.com/mbrobbel/narrow/commit/15a77e7cf48bac2b05e7ded072eb2f0e34e80c4a))
    - Remove bitvec dependency ([`e97195f`](https://github.com/mbrobbel/narrow/commit/e97195fd7571f221d063e26e8d6f6ff86ebda5a6))
    - Merge #24 ([`6ae74be`](https://github.com/mbrobbel/narrow/commit/6ae74be3c07d67173b25392165b53eb80fa908b9))
    - Fix Bitmap storage to match Arrow specification ([`543e8b2`](https://github.com/mbrobbel/narrow/commit/543e8b2666c6dd073333901986d67e02f0e03825))
    - Merge #21 ([`56ba436`](https://github.com/mbrobbel/narrow/commit/56ba436f08068e851c7a3a5e25ea869b09191e6e))
    - Fix unused_must_use warning ([`08d2918`](https://github.com/mbrobbel/narrow/commit/08d2918dff5cbd36b437edbd55675380fbdb4397))
    - Drop Clone bound for IntoIterator impl for NullArray ([`0b2fb93`](https://github.com/mbrobbel/narrow/commit/0b2fb93f9d96a1b795260b407c398795becc641f))
    - Fix clippy::len-without-is-empty for NullArray ([`7b84230`](https://github.com/mbrobbel/narrow/commit/7b842308af2fcef9f1eca48329579d8c153da613))
    - Modify NullArray to support Array derive for unit structs ([`e69d915`](https://github.com/mbrobbel/narrow/commit/e69d915340208e8b06b34ae6fc0aca6c98319d44))
    - Merge #20 ([`1649196`](https://github.com/mbrobbel/narrow/commit/1649196c7fb92c7302905a6cfeacb0889255a5c4))
    - Migrate to 2021 edition ([`60d723d`](https://github.com/mbrobbel/narrow/commit/60d723d3a721d8578ffe96ea857c7b9d124147ed))
</details>

## v0.1.0 (2021-07-29)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 16 commits contributed to the release over the course of 84 calendar days.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge #11 ([`4c4ead6`](https://github.com/mbrobbel/narrow/commit/4c4ead678350d72b069f8e817a094f0805b8c229))
    - Merge #12 ([`60fe6a9`](https://github.com/mbrobbel/narrow/commit/60fe6a9ff7fea980a3918ef601bbf57f69dfa800))
    - Include dictionary module ([`c87977d`](https://github.com/mbrobbel/narrow/commit/c87977dd9263f0e69f52721b566fb3c2fb3a6195))
    - Add DictionaryArray ([`6824088`](https://github.com/mbrobbel/narrow/commit/6824088b8c4ded9ee049aa587791d45c74c93981))
    - Add some comments ([`eccdee3`](https://github.com/mbrobbel/narrow/commit/eccdee3fd76a657e97f62f55831d9d6cb0c05510))
    - Add NullArray and UnionArray ([`ccd93d3`](https://github.com/mbrobbel/narrow/commit/ccd93d3d45163ba25cb1ddfaa9170e6c6233d22c))
    - Fix Clippy warning ([`6fa88e9`](https://github.com/mbrobbel/narrow/commit/6fa88e90b8b51b449307477675dc8e724ac117e1))
    - Add string array wrapper ([`baecd70`](https://github.com/mbrobbel/narrow/commit/baecd7089927381f62e1cd58043a570b515b377e))
    - Implement iterators for variable size array types ([`4f6e177`](https://github.com/mbrobbel/narrow/commit/4f6e177edef7187a0999fe88e40564474febaae3))
    - Add more Array types ([`02b9400`](https://github.com/mbrobbel/narrow/commit/02b9400edad3990099c99b72c1c0d73217e4bf9b))
    - Vec makes no guarantees about its memory layout ([`a95d324`](https://github.com/mbrobbel/narrow/commit/a95d3241b33b2fb34cbf1fa6ee63b12ba852ea40))
    - Add VariableSizeBinaryArray ([`916b520`](https://github.com/mbrobbel/narrow/commit/916b520584da266a0fee209224b5a12e9f21e2fe))
    - Add FixedSizeArray and BooleanArray ([`39e38f9`](https://github.com/mbrobbel/narrow/commit/39e38f941da1a27cecf543472a8846b05303b2ea))
    - Add Offset ([`b02eb45`](https://github.com/mbrobbel/narrow/commit/b02eb45ccec659d15dd0eca69771ca666f7943cc))
    - Add Buffer, Bitmap, Nullable and Validity ([`7d9df27`](https://github.com/mbrobbel/narrow/commit/7d9df27d1a309d75fbd79db46b1f2445b02d71a1))
    - Setup packages ([`355b72c`](https://github.com/mbrobbel/narrow/commit/355b72cdfcc9e48593efef252564b99066c9ebf9))
</details>

