

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

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#80](https://github.com/mbrobbel/narrow/issues/80)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#80](https://github.com/mbrobbel/narrow/issues/80)**
    - `ArrayType` derive for tuple structs ([`9a48422`](https://github.com/mbrobbel/narrow/commit/9a48422f4a8de0f9b5d109ce44c4c9a14544116a))
 * **Uncategorized**
    - Release narrow-derive v0.3.3, narrow v0.3.3 ([`cb07d58`](https://github.com/mbrobbel/narrow/commit/cb07d5839367936aa9b557a7b846ad7b9c98b7f2))
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

 - 4 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#79](https://github.com/mbrobbel/narrow/issues/79)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#79](https://github.com/mbrobbel/narrow/issues/79)**
    - `ArrayType` derive for unit structs ([`a7a3f79`](https://github.com/mbrobbel/narrow/commit/a7a3f79a98fc15879aabf677b17e12bb285ce57f))
 * **Uncategorized**
    - Release narrow-derive v0.3.2, narrow v0.3.2 ([`ac49ae4`](https://github.com/mbrobbel/narrow/commit/ac49ae411b3ca4021de805165225d0ce0e0801b6))
    - Merge branch 'main' into unit-struct-derive ([`09e3183`](https://github.com/mbrobbel/narrow/commit/09e31830dcbb6c34ca0905079bef9ca0ae15f317))
    - `ArrayType` derive for unit structs ([`e951ed1`](https://github.com/mbrobbel/narrow/commit/e951ed1510214d09794168f1b385289359b76b1c))
</details>

## v0.3.1 (2023-07-27)

<csr-id-17bf9944762a9b036fd6d1a5fa5280f2e68dba03/>

### Chore

 - <csr-id-17bf9944762a9b036fd6d1a5fa5280f2e68dba03/> fix gh release

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release narrow-derive v0.3.1, narrow v0.3.1 ([`902016c`](https://github.com/mbrobbel/narrow/commit/902016c0f3c44ce65c28183ea75d84ad173e29aa))
    - Fix gh release ([`17bf994`](https://github.com/mbrobbel/narrow/commit/17bf9944762a9b036fd6d1a5fa5280f2e68dba03))
</details>

## v0.3.0 (2023-07-27)

<csr-id-9a206988b5bf324e5279e552f9f336fb339db5ec/>
<csr-id-248e258f379d2f16a556cd182ee3d5641a926566/>
<csr-id-fbd7fb87ce887b92aa03d129f39a5405fef8c2fa/>

### Chore

 - <csr-id-9a206988b5bf324e5279e552f9f336fb339db5ec/> clear changelog
 - <csr-id-248e258f379d2f16a556cd182ee3d5641a926566/> attempt to fix release by bumping and removing changelogs
 - <csr-id-fbd7fb87ce887b92aa03d129f39a5405fef8c2fa/> attempt to fix release by bumping and removing changelogs

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#78](https://github.com/mbrobbel/narrow/issues/78)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#78](https://github.com/mbrobbel/narrow/issues/78)**
    - Attempt to fix release by bumping and removing changelogs ([`248e258`](https://github.com/mbrobbel/narrow/commit/248e258f379d2f16a556cd182ee3d5641a926566))
 * **Uncategorized**
    - Release narrow-derive v0.3.0, narrow v0.3.0 ([`2a5e4cc`](https://github.com/mbrobbel/narrow/commit/2a5e4cc2cb6bd00fd4ff177b2a32c7f5dd816b84))
    - Clear changelog ([`9a20698`](https://github.com/mbrobbel/narrow/commit/9a206988b5bf324e5279e552f9f336fb339db5ec))
    - Attempt to fix release by bumping and removing changelogs ([`fbd7fb8`](https://github.com/mbrobbel/narrow/commit/fbd7fb87ce887b92aa03d129f39a5405fef8c2fa))
</details>

## v0.2.2 (2023-07-26)

<csr-id-1b3b54e5df31ea737e04214a10840de61a06130a/>

### Chore

 - <csr-id-1b3b54e5df31ea737e04214a10840de61a06130a/> add changelogs

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 23 commits contributed to the release over the course of 810 calendar days.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#66](https://github.com/mbrobbel/narrow/issues/66)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#66](https://github.com/mbrobbel/narrow/issues/66)**
    - Add changelogs ([`1b3b54e`](https://github.com/mbrobbel/narrow/commit/1b3b54e5df31ea737e04214a10840de61a06130a))
 * **Uncategorized**
    - Release narrow-derive v0.2.2, narrow v0.2.2 ([`9466a73`](https://github.com/mbrobbel/narrow/commit/9466a730ed4fc10616a46b3abda422b3518d3fcc))
    - Add changelogs ([`142d01f`](https://github.com/mbrobbel/narrow/commit/142d01f059ba1c0ff112004fc90f4255141c6152))
    - Merge #62 ([`5deccd3`](https://github.com/mbrobbel/narrow/commit/5deccd3dfd6e1e7d05566c8db5fe6aff52ba6072))
    - Change ~some~ all things ([`952ccf0`](https://github.com/mbrobbel/narrow/commit/952ccf0cbc69d18a3624dc3699a57905bfeb18be))
    - Merge #58 #59 ([`db768b5`](https://github.com/mbrobbel/narrow/commit/db768b52a2c40920dd6b976277af9c174317fda9))
    - Update syn requirement from 1 to 2 ([`e287ba1`](https://github.com/mbrobbel/narrow/commit/e287ba15d937f6a0e0bae5454a9bff325a89a711))
    - Merge #47 ([`f4e88e6`](https://github.com/mbrobbel/narrow/commit/f4e88e65cee6e01dcd03485789465d2af2f31244))
    - Fix syn features ([`807c86a`](https://github.com/mbrobbel/narrow/commit/807c86a19884188790b2121c1ae7b8ee4f23e311))
    - Merge #40 ([`69d6758`](https://github.com/mbrobbel/narrow/commit/69d6758f40cf521f0964741a47e9eb725f40c5cf))
    - Compile using stable Rust ([`74cdd1f`](https://github.com/mbrobbel/narrow/commit/74cdd1f77443a33456e832b2ab47f07edf5945d4))
    - Merge #28 ([`51ac77b`](https://github.com/mbrobbel/narrow/commit/51ac77b24414046aaa095b75090d8216ca520f69))
    - Starting afresh ([`aa7f5c2`](https://github.com/mbrobbel/narrow/commit/aa7f5c2881006a8c79c9716f62c0dfeb9387405a))
    - Merge #20 ([`1649196`](https://github.com/mbrobbel/narrow/commit/1649196c7fb92c7302905a6cfeacb0889255a5c4))
    - Migrate to 2021 edition ([`60d723d`](https://github.com/mbrobbel/narrow/commit/60d723d3a721d8578ffe96ea857c7b9d124147ed))
    - Fix license fields in manifests ([`c3e13e7`](https://github.com/mbrobbel/narrow/commit/c3e13e72fc9719676eb24d3a97cc77f2bb5a5be1))
    - Setup release workflow ([`7de0ac1`](https://github.com/mbrobbel/narrow/commit/7de0ac129c933e25760b2a48e14f707771f24b50))
    - Merge #11 ([`4c4ead6`](https://github.com/mbrobbel/narrow/commit/4c4ead678350d72b069f8e817a094f0805b8c229))
    - Add some comments ([`eccdee3`](https://github.com/mbrobbel/narrow/commit/eccdee3fd76a657e97f62f55831d9d6cb0c05510))
    - Add NullArray and UnionArray ([`ccd93d3`](https://github.com/mbrobbel/narrow/commit/ccd93d3d45163ba25cb1ddfaa9170e6c6233d22c))
    - Add string array wrapper ([`baecd70`](https://github.com/mbrobbel/narrow/commit/baecd7089927381f62e1cd58043a570b515b377e))
    - Add more Array types ([`02b9400`](https://github.com/mbrobbel/narrow/commit/02b9400edad3990099c99b72c1c0d73217e4bf9b))
    - Setup packages ([`355b72c`](https://github.com/mbrobbel/narrow/commit/355b72cdfcc9e48593efef252564b99066c9ebf9))
</details>

