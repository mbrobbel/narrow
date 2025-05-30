//! Array with fixed-size sequences of elements.

use std::{
    iter::{self, Map, Zip},
    mem::{self, ManuallyDrop, MaybeUninit},
};

use crate::{
    Index, Length,
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{BufferMut, BufferType, VecBuffer},
    nullability::{NonNullable, Nullability, Nullable},
    validity::Validity,
};

use super::Array;

/// Array with fixed-size sequences of elements.
pub struct FixedSizeListArray<
    const N: usize,
    T: Array,
    Nullable: Nullability = NonNullable,
    Buffer: BufferType = VecBuffer,
>(pub(crate) Nullable::Collection<T, Buffer>);

impl<const N: usize, T: Array + Length, Nullable: Nullability, Buffer: BufferType>
    FixedSizeListArray<N, T, Nullable, Buffer>
where
    FixedSizeListArray<N, T, Nullable, Buffer>: Index + Length,
{
    /// Returns an iterator over items in this [`FixedSizeListArray`].
    pub fn iter(&self) -> FixedSizeListIter<'_, N, T, Nullable, Buffer> {
        <&Self as IntoIterator>::into_iter(self)
    }
}

impl<const N: usize, T: Array, Nullable: Nullability, Buffer: BufferType> Array
    for FixedSizeListArray<N, T, Nullable, Buffer>
{
    type Item = Nullable::Item<[<T as Array>::Item; N]>;
}

impl<const N: usize, T: Array, Buffer: BufferType> BitmapRef
    for FixedSizeListArray<N, T, Nullable, Buffer>
{
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<const N: usize, T: Array, Buffer: BufferType> BitmapRefMut
    for FixedSizeListArray<N, T, Nullable, Buffer>
{
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<const N: usize, T: Array, Nullable: Nullability, Buffer: BufferType> Clone
    for FixedSizeListArray<N, T, Nullable, Buffer>
where
    Nullable::Collection<T, Buffer>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<const N: usize, T: Array, Nullable: Nullability, Buffer: BufferType> Default
    for FixedSizeListArray<N, T, Nullable, Buffer>
where
    Nullable::Collection<T, Buffer>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<U, const N: usize, T: Array, Buffer: BufferType> Extend<[U; N]>
    for FixedSizeListArray<N, T, NonNullable, Buffer>
where
    T: Extend<U>,
{
    fn extend<I: IntoIterator<Item = [U; N]>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter().flatten());
    }
}

impl<U, const N: usize, T: Array, Buffer: BufferType> Extend<Option<[U; N]>>
    for FixedSizeListArray<N, T, Nullable, Buffer>
where
    [U; N]: Default,
    T: Extend<U>,
    Bitmap<Buffer>: Extend<bool>,
{
    fn extend<I: IntoIterator<Item = Option<[U; N]>>>(&mut self, iter: I) {
        self.0.data.extend(
            iter.into_iter()
                .inspect(|opt| {
                    self.0.validity.extend(iter::once(opt.is_some()));
                })
                .flat_map(Option::unwrap_or_default),
        );
    }
}

impl<const N: usize, T: Array, Buffer: BufferType>
    From<FixedSizeListArray<N, T, NonNullable, Buffer>>
    for FixedSizeListArray<N, T, Nullable, Buffer>
where
    T: Length,
    Bitmap<Buffer>: FromIterator<bool>,
{
    fn from(value: FixedSizeListArray<N, T, NonNullable, Buffer>) -> Self {
        Self(Validity::from(value.0))
    }
}

impl<U, const N: usize, T: Array, Buffer: BufferType> FromIterator<[U; N]>
    for FixedSizeListArray<N, T, NonNullable, Buffer>
where
    T: FromIterator<U>,
{
    fn from_iter<I: IntoIterator<Item = [U; N]>>(iter: I) -> Self {
        Self(iter.into_iter().flatten().collect())
    }
}

impl<U, const N: usize, T: Array, Buffer: BufferType> FromIterator<Option<[U; N]>>
    for FixedSizeListArray<N, T, Nullable, Buffer>
where
    [U; N]: Default,
    T: FromIterator<U>,
    Buffer::Buffer<u8>: Default + BufferMut<u8> + Extend<u8>,
{
    fn from_iter<I: IntoIterator<Item = Option<[U; N]>>>(iter: I) -> Self {
        let mut validity = Bitmap::default();
        let data = iter
            .into_iter()
            .inspect(|opt| {
                validity.extend(iter::once(opt.is_some()));
            })
            .flat_map(Option::unwrap_or_default)
            .collect();
        Self(Validity { data, validity })
    }
}

impl<const N: usize, T: Array, Buffer: BufferType> Index
    for FixedSizeListArray<N, T, NonNullable, Buffer>
where
    T: Index,
{
    type Item<'a>
        = [<T as Index>::Item<'a>; N]
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        // Following https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        let mut data: [MaybeUninit<_>; N] = MaybeUninit::uninit().assume_init();
        let start_index = index * N;
        let end_index = start_index + N;
        (start_index..end_index)
            .enumerate()
            .for_each(|(array_index, child_index)| {
                data[array_index].write(self.0.index_unchecked(child_index));
            });
        // https://github.com/rust-lang/rust/issues/61956
        mem::transmute_copy(&ManuallyDrop::new(data))
    }
}

impl<const N: usize, T: Array, Buffer: BufferType> Index
    for FixedSizeListArray<N, T, Nullable, Buffer>
where
    T: Index,
{
    type Item<'a>
        = Option<[<T as Index>::Item<'a>; N]>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.is_valid_unchecked(index).then(|| {
            // Following https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
            let mut data: [MaybeUninit<_>; N] = MaybeUninit::uninit().assume_init();
            let start_index = index * N;
            let end_index = start_index + N;
            (start_index..end_index)
                .enumerate()
                .for_each(|(array_index, child_index)| {
                    // Here we need to index in the data
                    data[array_index].write(self.0.data.index_unchecked(child_index));
                });
            // https://github.com/rust-lang/rust/issues/61956
            mem::transmute_copy(&ManuallyDrop::new(data))
        })
    }
}

/// An iterator over fixed-size lists in a [`FixedSizeListArray`].
pub struct FixedSizeListIter<
    'a,
    const N: usize,
    T: Array,
    Nullable: Nullability,
    Buffer: BufferType,
> {
    /// Reference to the array.
    array: &'a FixedSizeListArray<N, T, Nullable, Buffer>,
    /// Current index.
    index: usize,
}

impl<'a, const N: usize, T: Array, Nullable: Nullability, Buffer: BufferType> Iterator
    for FixedSizeListIter<'a, N, T, Nullable, Buffer>
where
    FixedSizeListArray<N, T, Nullable, Buffer>: Length + Index,
{
    type Item = <FixedSizeListArray<N, T, Nullable, Buffer> as Index>::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.array
            .index(self.index)
            .into_iter()
            .inspect(|_| {
                self.index += 1;
            })
            .next()
    }
}

impl<'a, const N: usize, T: Array, Nullable: Nullability, Buffer: BufferType> IntoIterator
    for &'a FixedSizeListArray<N, T, Nullable, Buffer>
where
    FixedSizeListArray<N, T, Nullable, Buffer>: Index + Length,
{
    type Item = <FixedSizeListArray<N, T, Nullable, Buffer> as Index>::Item<'a>;
    type IntoIter = FixedSizeListIter<'a, N, T, Nullable, Buffer>;

    fn into_iter(self) -> Self::IntoIter {
        FixedSizeListIter {
            array: self,
            index: 0,
        }
    }
}

/// An iterator over `N` elements of the iterator at a time.
pub struct FixedSizeArrayChunks<const N: usize, I: Iterator> {
    /// An owned iterator
    iter: I,
}

impl<const N: usize, I: Iterator> FixedSizeArrayChunks<N, I> {
    /// Returns a new [`FixedSizeArrayChunks`]
    fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<const N: usize, I: Iterator> Iterator for FixedSizeArrayChunks<N, I> {
    type Item = [I::Item; N];

    fn next(&mut self) -> Option<Self::Item> {
        let mut data: [MaybeUninit<I::Item>; N] =
        // Safety:
        // - https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        unsafe { MaybeUninit::uninit().assume_init() };

        let mut total_elements_written: usize = 0;
        self.iter
            .by_ref()
            .take(N)
            .enumerate()
            .for_each(|(array_index, val)| {
                data[array_index].write(val);
                total_elements_written += 1;
            });

        assert!(total_elements_written <= N);

        if total_elements_written == N {
            Some(data.map(|elem| {
                // Safety:
                // - We only initialize if we acually wrote to this element.
                unsafe { elem.assume_init() }
            }))
        } else {
            // For each elem in the array, drop if we wrote to it to prevent memory leaks.
            for elem in &mut data[0..total_elements_written] {
                // Safety:
                // - This element was initialized as indicated by `total_elements_written`.
                unsafe {
                    // Drop the value to prevent a memory leak.
                    elem.assume_init_drop();
                }
            }
            None
        }
    }
}

impl<const N: usize, T: Array, Buffer: BufferType> IntoIterator
    for FixedSizeListArray<N, T, NonNullable, Buffer>
where
    T: IntoIterator,
    FixedSizeArrayChunks<N, <T as IntoIterator>::IntoIter>:
        IntoIterator<Item = [<T as IntoIterator>::Item; N]>,
{
    type Item = [<T as IntoIterator>::Item; N];
    type IntoIter = FixedSizeArrayChunks<N, <T as IntoIterator>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        FixedSizeArrayChunks::<N, _>::new(self.0.into_iter())
    }
}

impl<const N: usize, T: Array, Buffer: BufferType> IntoIterator
    for FixedSizeListArray<N, T, Nullable, Buffer>
where
    T: IntoIterator,
    Bitmap<Buffer>: IntoIterator<Item = bool>,
    FixedSizeArrayChunks<N, <T as IntoIterator>::IntoIter>:
        IntoIterator<Item = [<T as IntoIterator>::Item; N]>,
{
    type Item = Option<[<T as IntoIterator>::Item; N]>;
    type IntoIter = Map<
        Zip<
            <Bitmap<Buffer> as IntoIterator>::IntoIter,
            <FixedSizeArrayChunks<N, <T as IntoIterator>::IntoIter> as IntoIterator>::IntoIter,
        >,
        fn((bool, [<T as IntoIterator>::Item; N])) -> Self::Item,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.0
            .validity
            .into_iter()
            .zip(FixedSizeArrayChunks::<N, _>::new(self.0.data.into_iter()))
            .map(|(validity, value)| validity.then_some(value))
    }
}

impl<const N: usize, T: Array, Nullable: Nullability, Buffer: BufferType> Length
    for FixedSizeListArray<N, T, Nullable, Buffer>
where
    Nullable::Collection<T, Buffer>: Length,
{
    fn len(&self) -> usize {
        if Nullable::NULLABLE {
            // This uses the length of the validity bitmap
            self.0.len()
        } else {
            self.0.len() / N
        }
    }
}

impl<const N: usize, T: Array, Buffer: BufferType> ValidityBitmap
    for FixedSizeListArray<N, T, Nullable, Buffer>
{
}

#[cfg(test)]
mod tests {
    use crate::array::{FixedSizePrimitiveArray, StringArray};

    use super::*;

    #[test]
    fn from_iter() {
        {
            let input_non_nullable = [[1_u8, 2], [3, 4]];
            let array_non_nullable = input_non_nullable
                .into_iter()
                .collect::<FixedSizeListArray<2, FixedSizePrimitiveArray<u8>>>();
            assert_eq!(array_non_nullable.len(), 2);
        };

        {
            let input_inner_nullable = [[Some(1_u8), None], [Some(3), None]];
            let array_inner_nullable = input_inner_nullable
                .into_iter()
                .collect::<FixedSizeListArray<2, FixedSizePrimitiveArray<u8, Nullable>, NonNullable>>();
            assert_eq!(array_inner_nullable.len(), 2);
            assert_eq!(
                array_inner_nullable.into_iter().collect::<Vec<_>>(),
                input_inner_nullable
            );
        };

        {
            let input_outer_nullable = [Some([1_u8, 1_u8]), Some([1_u8, 1_u8]), None];
            let array_outer_nullable = input_outer_nullable
                .into_iter()
                .collect::<FixedSizeListArray<2, FixedSizePrimitiveArray<u8, NonNullable>, Nullable>>();
            assert_eq!(array_outer_nullable.len(), 3);
            assert_eq!(
                array_outer_nullable.into_iter().collect::<Vec<_>>(),
                input_outer_nullable
            );
        };

        {
            let input_both_nullable = [Some([Some(1_u8), None]), None];
            let array_both_nullable = input_both_nullable
                .into_iter()
                .collect::<FixedSizeListArray<2, FixedSizePrimitiveArray<u8, Nullable>, Nullable>>(
                );
            assert_eq!(array_both_nullable.len(), 2);
            assert_eq!(
                array_both_nullable.into_iter().collect::<Vec<_>>(),
                input_both_nullable
            );
        };

        {
            let input_nested_innermost_nullable = [
                [
                    [Some(1_u8), None, Some(1_u8)],
                    [Some(3_u8), None, Some(1_u8)],
                ],
                [
                    [Some(2_u8), None, Some(1_u8)],
                    [Some(5_u8), None, Some(1_u8)],
                ],
            ];
            let array_nested_innermost_nullable = input_nested_innermost_nullable
                .into_iter()
                .collect::<FixedSizeListArray<
                2,
                FixedSizeListArray<3, FixedSizePrimitiveArray<u8, Nullable>, NonNullable>,
                NonNullable,
            >>();
            assert_eq!(array_nested_innermost_nullable.len(), 2);
            assert_eq!(
                array_nested_innermost_nullable
                    .into_iter()
                    .collect::<Vec<_>>(),
                input_nested_innermost_nullable
            );
        };

        {
            let input_nested_all_nullable = [
                None,
                Some([
                    None,
                    Some([Some(1_u8), None, Some(2)]),
                    Some([Some(3), None, Some(1)]),
                    None,
                ]),
                Some([
                    Some([Some(2), None, Some(1)]),
                    None,
                    None,
                    Some([Some(5), None, Some(6)]),
                ]),
            ];
            let array_nested_all_nullable = input_nested_all_nullable
                .into_iter()
                .collect::<FixedSizeListArray<
                    4,
                    FixedSizeListArray<3, FixedSizePrimitiveArray<u8, Nullable>, Nullable>,
                    Nullable,
                >>();
            assert_eq!(array_nested_all_nullable.len(), 3);
            assert_eq!(
                array_nested_all_nullable.into_iter().collect::<Vec<_>>(),
                input_nested_all_nullable
            );
        };
    }

    #[test]
    fn from_iter_variable_size() {
        {
            let input_string_non_nullable = [
                ["hello".to_owned(), "world".to_owned()],
                ["!".to_owned(), "!".to_owned()],
            ];
            let array_string_non_nullable = input_string_non_nullable
                .clone()
                .into_iter()
                .collect::<FixedSizeListArray<2, StringArray>>();
            assert_eq!(array_string_non_nullable.len(), 2);
            assert_eq!(
                array_string_non_nullable.into_iter().collect::<Vec<_>>(),
                input_string_non_nullable
            );
        };

        {
            let input_string_nested_all_nullable = [
                None,
                Some([
                    Some([Some("hello".to_owned()), None, Some("from".to_owned())]),
                    Some([Some("the".to_owned()), None, Some("other".to_owned())]),
                    None,
                    None,
                ]),
                Some([
                    None,
                    Some([Some("side".to_owned()), None, Some("hello".to_owned())]),
                    None,
                    Some([Some("its".to_owned()), None, Some("me".to_owned())]),
                ]),
            ];
            let array_string_nested_all_nullable = input_string_nested_all_nullable
                .clone()
                .into_iter()
                .collect::<FixedSizeListArray<
                    4,
                    FixedSizeListArray<3, StringArray<Nullable>, Nullable>,
                    Nullable,
                >>();
            assert_eq!(array_string_nested_all_nullable.len(), 3);
            assert_eq!(
                array_string_nested_all_nullable
                    .into_iter()
                    .collect::<Vec<_>>(),
                input_string_nested_all_nullable
            );
        };

        {
            let input_string_even_more_nested = [
                Some([
                    Some([Some(["hello".to_owned()]), None, Some(["from".to_owned()])]),
                    Some([Some(["the".to_owned()]), None, Some(["other".to_owned()])]),
                    None,
                    None,
                ]),
                None,
                Some([
                    None,
                    Some([Some(["side".to_owned()]), None, Some(["hello".to_owned()])]),
                    None,
                    Some([Some(["its".to_owned()]), None, Some(["me".to_owned()])]),
                ]),
            ];
            let array_string_even_more_nested = input_string_even_more_nested
                .clone()
                .into_iter()
                .collect::<FixedSizeListArray<
                4,
                FixedSizeListArray<
                    3,
                    FixedSizeListArray<1, StringArray<NonNullable>, Nullable>,
                    Nullable,
                >,
                Nullable,
            >>();
            assert_eq!(array_string_even_more_nested.len(), 3);
            assert_eq!(
                array_string_even_more_nested
                    .into_iter()
                    .collect::<Vec<_>>(),
                input_string_even_more_nested
            );
        };
    }

    #[test]
    fn index() {
        let input = [[1_u8, 2], [3, 4]];
        let array = input
            .into_iter()
            .collect::<FixedSizeListArray<2, FixedSizePrimitiveArray<u8>>>();
        assert_eq!(array.index(0), Some([&1, &2]));
        assert_eq!(array.index(1), Some([&3, &4]));

        let input_string = [["hello", "world"], ["!", "!"]];
        let array_string = input_string
            .into_iter()
            .collect::<FixedSizeListArray<2, StringArray>>();
        assert_eq!(array_string.index(0), Some(["hello", "world"]));
        assert_eq!(array_string.index(1), Some(["!", "!"]));

        let input_nullable_string = [Some(["hello", "world"]), None];
        let array_nullable_string = input_nullable_string
            .into_iter()
            .collect::<FixedSizeListArray<2, StringArray, Nullable>>();
        assert_eq!(
            array_nullable_string.index(0),
            Some(Some(["hello", "world"]))
        );
        assert_eq!(array_nullable_string.index(1), Some(None));
        assert_eq!(array_nullable_string.index(2), None);

        let input_nullable_string_nullable = [
            Some([Some("hello"), None]),
            None,
            Some([None, Some("world")]),
        ];
        let array_nullable_string_nullable = input_nullable_string_nullable
            .into_iter()
            .collect::<FixedSizeListArray<2, StringArray<Nullable>, Nullable>>(
        );
        assert_eq!(
            array_nullable_string_nullable.index(0),
            Some(Some([Some("hello"), None]))
        );
        assert_eq!(array_nullable_string_nullable.index(1), Some(None));
        assert_eq!(
            array_nullable_string_nullable.index(2),
            Some(Some([None, Some("world")]))
        );
        assert_eq!(array_nullable_string_nullable.index(3), None);
    }

    #[test]
    fn fixed_size_array_chunks() {
        {
            let input = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
            let array_chunks = FixedSizeArrayChunks::<3, _>::new(input.into_iter());
            assert_eq!(
                array_chunks.into_iter().collect::<Vec<_>>(),
                vec![[0, 1, 2], [3, 4, 5], [6, 7, 8]]
            );
        };

        {
            // only returns complete chunks.
            let input = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
            let array_chunks = FixedSizeArrayChunks::<3, _>::new(input.into_iter());
            assert_eq!(
                array_chunks.into_iter().collect::<Vec<_>>(),
                vec![[0, 1, 2], [3, 4, 5], [6, 7, 8]]
            );
        }
    }

    #[test]
    fn into_iter() {
        let input = [[1_u8, 2], [3, 4]];
        let array = input
            .into_iter()
            .collect::<FixedSizeListArray<2, FixedSizePrimitiveArray<u8>>>();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), [[1, 2], [3, 4]]);

        let input_string = [["hello", "world"], ["!", "!"]];
        let array_string = input_string
            .into_iter()
            .collect::<FixedSizeListArray<2, StringArray>>();
        assert_eq!(array_string.into_iter().collect::<Vec<_>>(), input_string);

        let input_nullable_string = [
            Some(["hello".to_owned(), "world".to_owned()]),
            None,
            Some(["hello".to_owned(), "again".to_owned()]),
        ];
        let array_nullable_string = input_nullable_string
            .clone()
            .into_iter()
            .collect::<FixedSizeListArray<2, StringArray, Nullable>>();
        assert_eq!(
            array_nullable_string.into_iter().collect::<Vec<_>>(),
            input_nullable_string
        );

        let input_nullable_string_nullable = [
            Some([Some("hello".to_owned()), None]),
            None,
            Some([None, Some("world".to_owned())]),
            None,
            Some([Some("hello".to_owned()), Some("again".to_owned())]),
        ];
        let array_nullable_string_nullable = input_nullable_string_nullable
            .clone()
            .into_iter()
            .collect::<FixedSizeListArray<2, StringArray<Nullable>, Nullable>>(
        );
        assert_eq!(
            array_nullable_string_nullable
                .into_iter()
                .collect::<Vec<_>>(),
            input_nullable_string_nullable
        );

        let input_nested = [[[1, 2], [3, 4], [5, 6]], [[7, 8], [9, 0], [0, 0]]];
        let array_nested = input_nested
            .into_iter()
            .collect::<FixedSizeListArray<3, FixedSizeListArray<2, FixedSizePrimitiveArray<u8>>>>();

        assert_eq!(array_nested.0.0.0, [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0, 0]);
        assert_eq!(
            array_nested.into_iter().collect::<Vec<_>>(),
            [[[1, 2], [3, 4], [5, 6]], [[7, 8], [9, 0], [0, 0]]]
        );
    }
}
