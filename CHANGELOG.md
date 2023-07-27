

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

