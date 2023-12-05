//! Array with fixed-size sequences of elements.

use std::{
    iter,
    mem::{self, ManuallyDrop, MaybeUninit},
};

use crate::{
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{BufferMut, BufferType, VecBuffer},
    nullable::Nullable,
    validity::Validity,
    Index, Length,
};

use super::Array;

/// Array with fixed-size sequences of elements.
pub struct FixedSizeListArray<
    const N: usize,
    T: Array,
    const NULLABLE: bool = false,
    Buffer: BufferType = VecBuffer,
>(<T as Validity<NULLABLE>>::Storage<Buffer>)
where
    T: Validity<NULLABLE>;

impl<const N: usize, T: Array, const NULLABLE: bool, Buffer: BufferType> Array
    for FixedSizeListArray<N, T, NULLABLE, Buffer>
where
    T: Validity<NULLABLE>,
{
}

impl<const N: usize, T: Array, Buffer: BufferType> BitmapRef
    for FixedSizeListArray<N, T, true, Buffer>
{
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<const N: usize, T: Array, Buffer: BufferType> BitmapRefMut
    for FixedSizeListArray<N, T, true, Buffer>
{
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<const N: usize, T: Array, const NULLABLE: bool, Buffer: BufferType> Default
    for FixedSizeListArray<N, T, NULLABLE, Buffer>
where
    T: Validity<NULLABLE>,
    <T as Validity<NULLABLE>>::Storage<Buffer>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<U, const N: usize, T: Array, const NULLABLE: bool, Buffer: BufferType> Extend<U>
    for FixedSizeListArray<N, T, NULLABLE, Buffer>
where
    T: Validity<NULLABLE>,
    <T as Validity<NULLABLE>>::Storage<Buffer>: Extend<U>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<const N: usize, T: Array, Buffer: BufferType> From<FixedSizeListArray<N, T, false, Buffer>>
    for FixedSizeListArray<N, T, true, Buffer>
where
    T: Length,
    Bitmap<Buffer>: FromIterator<bool>,
{
    fn from(value: FixedSizeListArray<N, T, false, Buffer>) -> Self {
        Self(Nullable::from(value.0))
    }
}

impl<U, const N: usize, T: Array, Buffer: BufferType> FromIterator<[U; N]>
    for FixedSizeListArray<N, T, false, Buffer>
where
    T: FromIterator<U>,
{
    fn from_iter<I: IntoIterator<Item = [U; N]>>(iter: I) -> Self {
        Self(iter.into_iter().flatten().collect())
    }
}

impl<U, const N: usize, T: Array, Buffer: BufferType> FromIterator<Option<[U; N]>>
    for FixedSizeListArray<N, T, true, Buffer>
where
    [U; N]: Default,
    T: FromIterator<U>,
    <Buffer as BufferType>::Buffer<u8>: Default + BufferMut<u8> + Extend<u8>,
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
        Self(Nullable { data, validity })
    }
}

impl<const N: usize, T: Array, Buffer: BufferType> Index for FixedSizeListArray<N, T, false, Buffer>
where
    T: Index,
{
    type Item<'a> = [<T as Index>::Item<'a>; N]
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        // Following https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        let data = {
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
        };
        data
    }
}

impl<const N: usize, T: Array, Buffer: BufferType> Index for FixedSizeListArray<N, T, true, Buffer>
where
    T: Index,
{
    type Item<'a> = Option<[<T as Index>::Item<'a>; N]>
    where
        Self: 'a;

    unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
        self.is_valid_unchecked(index).then(|| {
            // Following https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
            let data = {
                let mut data: [MaybeUninit<_>; N] = MaybeUninit::uninit().assume_init();
                let start_index = index * N;
                let end_index = start_index + N;
                (start_index..end_index)
                    .enumerate()
                    .for_each(|(array_index, child_index)| {
                        data[array_index].write(self.0.data.index_unchecked(child_index));
                    });
                // https://github.com/rust-lang/rust/issues/61956
                mem::transmute_copy(&ManuallyDrop::new(data))
            };
            data
        })
    }
}

impl<const N: usize, T: Array, const NULLABLE: bool, Buffer: BufferType> Length
    for FixedSizeListArray<N, T, NULLABLE, Buffer>
where
    T: Validity<NULLABLE>,
    <T as Validity<NULLABLE>>::Storage<Buffer>: Length,
{
    fn len(&self) -> usize {
        if NULLABLE {
            // This uses the length of the validity bitmap
            self.0.len()
        } else {
            self.0.len() / N
        }
    }
}

impl<const N: usize, T: Array, Buffer: BufferType> ValidityBitmap
    for FixedSizeListArray<N, T, true, Buffer>
{
}

#[cfg(test)]
mod tests {
    use crate::array::{FixedSizePrimitiveArray, StringArray};

    use super::*;

    #[test]
    fn from_iter() {
        let input = [[1_u8, 2], [3, 4]];
        let array = input
            .into_iter()
            .collect::<FixedSizeListArray<2, FixedSizePrimitiveArray<u8>>>();
        assert_eq!(array.len(), 2);

        let input_nullable = [Some([1_u8, 2]), None];
        let array_nullable =
            input_nullable
                .into_iter()
                .collect::<FixedSizeListArray<2, FixedSizePrimitiveArray<u8>, true>>();
        assert_eq!(array_nullable.len(), 2);

        let input_string = [["hello", "world"], ["!", "!"]];
        let array_string = input_string
            .into_iter()
            .collect::<FixedSizeListArray<2, StringArray>>();
        assert_eq!(array_string.len(), 2);
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
            .collect::<FixedSizeListArray<2, StringArray, true>>();
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
            .collect::<FixedSizeListArray<2, StringArray<true>, true>>(
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
}
