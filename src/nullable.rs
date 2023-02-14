//! Nullable data.

use std::iter::{Map, Zip};

use crate::{
    bitmap::{
        iter::{BitmapIntoIter, BitmapIter},
        Bitmap, ValidityBitmap,
    },
    buffer::{Buffer, BufferAlloc, BufferExtend, BufferRef, BufferRefMut, BufferTake},
    Length,
};

/// Wrapper for nullable data.
///
/// Store data with a validity [Bitmap] that uses a single bit per value in `T`
/// that indicates the nullness or non-nullness of that value.
pub struct Nullable<DataBuffer, BitmapBuffer = Vec<u8>>
// where
//     BitmapBuffer: Buffer<u8>,
{
    /// Data that could contain null elements.
    data: DataBuffer,

    // TODO(mbrobbel): wrap Bitmap in Option to handle external data for nullable types that don't
    // have a validity buffer allocated. None indicates all the values in T are valid.
    /// The validity bitmap with validity information for the elements in the
    /// data.
    validity: Bitmap<BitmapBuffer>,
}

impl<DataBuffer, BitmapBuffer> BufferRef for Nullable<DataBuffer, BitmapBuffer>
where
    DataBuffer: BufferRef,
{
    type Buffer = <DataBuffer as BufferRef>::Buffer;
    type Element = <DataBuffer as BufferRef>::Element;

    fn buffer_ref(&self) -> &Self::Buffer {
        self.data.buffer_ref()
    }
}

impl<DataBuffer, BitmapBuffer> BufferRefMut for Nullable<DataBuffer, BitmapBuffer>
where
    DataBuffer: BufferRefMut,
{
    type BufferMut = <DataBuffer as BufferRefMut>::BufferMut;
    type Element = <DataBuffer as BufferRefMut>::Element;

    fn buffer_ref_mut(&mut self) -> &mut Self::BufferMut {
        self.data.buffer_ref_mut()
    }
}

impl<T, U, Data, BitmapBuffer> FromIterator<(bool, U)> for Nullable<Data, BitmapBuffer>
where
    T: Default,
    U: IntoIterator<Item = T>,
    Data: Default + Extend<T>,
    BitmapBuffer: BufferAlloc<u8>,
{
    fn from_iter<I: IntoIterator<Item = (bool, U)>>(iter: I) -> Self {
        let mut data = Data::default();
        data.extend(Some(T::default()));
        let validity = iter
            .into_iter()
            .map(|(valid, item)| {
                data.extend(item);
                valid
            })
            .collect();
        Self { data, validity }
    }
}

impl<T, DataBuffer, BitmapBuffer> FromIterator<Option<T>> for Nullable<DataBuffer, BitmapBuffer>
where
    T: Default,
    DataBuffer: Default + Extend<T>,
    BitmapBuffer: Default + BufferExtend<u8>,
{
    fn from_iter<I: IntoIterator<Item = Option<T>>>(iter: I) -> Self {
        let (validity, data) = iter
            .into_iter()
            .map(|opt| (opt.is_some(), opt.unwrap_or_default()))
            .unzip();
        Self { data, validity }
    }
}

impl<DataBuffer, BitmapBuffer> IntoIterator for Nullable<DataBuffer, BitmapBuffer>
where
    DataBuffer: IntoIterator,
    BitmapBuffer: BufferTake<u8>,
{
    type IntoIter = Map<
        Zip<
            BitmapIntoIter<<BitmapBuffer as IntoIterator>::IntoIter>,
            <DataBuffer as IntoIterator>::IntoIter,
        >,
        fn((bool, <DataBuffer as IntoIterator>::Item)) -> Self::Item,
    >;
    type Item = Option<<DataBuffer as IntoIterator>::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.validity
            .into_iter()
            .zip(self.data.into_iter())
            .map(|(validity, value)| validity.then_some(value))
    }
}

impl<'a, DataBuffer, BitmapBuffer> IntoIterator for &'a Nullable<DataBuffer, BitmapBuffer>
where
    &'a DataBuffer: IntoIterator,
    BitmapBuffer: Buffer<u8>,
{
    type IntoIter = Map<
        Zip<BitmapIter<'a>, <&'a DataBuffer as IntoIterator>::IntoIter>,
        fn((bool, <&'a DataBuffer as IntoIterator>::Item)) -> Self::Item,
    >;
    type Item = Option<<&'a DataBuffer as IntoIterator>::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.validity
            .into_iter()
            .zip(self.data.into_iter())
            .map(|(validity, value)| validity.then_some(value))
    }
}

impl<DataBuffer, BitmapBuffer> ValidityBitmap for Nullable<DataBuffer, BitmapBuffer>
where
    BitmapBuffer: Buffer<u8>,
{
    type Buffer = BitmapBuffer;

    #[inline]
    fn validity_bitmap(&self) -> &Bitmap<BitmapBuffer> {
        &self.validity
    }
}

impl<DataBuffer, BitmapBuffer> Length for Nullable<DataBuffer, BitmapBuffer> {
    fn len(&self) -> usize {
        self.validity.len()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        iter::{self, Repeat, Take},
        mem,
    };

    use super::*;
    use crate::buffer::BufferRef;

    #[test]
    fn from_iter() {
        let input = [Some(1u32), Some(2), Some(3), Some(4), None, Some(42)];
        let nullable = input.into_iter().collect::<Nullable<Vec<_>>>();
        assert_eq!(nullable.buffer_ref(), &[1, 2, 3, 4, u32::default(), 42]);
        assert_eq!(nullable.validity_bitmap().buffer_ref(), &[0b00101111u8]);

        let input = [
            Some([1, 1]),
            Some([2, 2]),
            Some([3, 3]),
            Some([4, 4]),
            None,
            Some([42, 42]),
        ];
        let nullable = input.into_iter().collect::<Nullable<Vec<_>>>();
        assert_eq!(
            nullable.buffer_ref(),
            &[
                1,
                1,
                2,
                2,
                3,
                3,
                4,
                4,
                u32::default(),
                u32::default(),
                42,
                42
            ]
        );
        assert_eq!(nullable.validity_bitmap().buffer_ref(), &[0b00101111u8]);
    }

    #[test]
    fn into_iter() {
        let input = [Some(1u32), Some(2), Some(3), Some(4), None, Some(42)];
        let nullable = input.into_iter().collect::<Nullable<Vec<_>>>();
        let output = nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());
    }

    #[test]
    fn opt_bool_iter() {
        let input = [Some(true), Some(false), None];
        let nullable = input.into_iter().collect::<Nullable<Bitmap>>();
        assert_eq!(nullable.buffer_ref(), &[0b00000001u8]);
        assert_eq!(nullable.validity_bitmap().buffer_ref(), &[0b00000011u8]);
    }

    #[test]
    fn count_iter() {
        #[derive(Default)]
        struct Count(usize);

        impl<T> FromIterator<T> for Count {
            fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
                Self(iter.into_iter().count())
            }
        }

        impl<T> Extend<T> for Count {
            fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
                self.0 += iter.into_iter().count();
            }
        }

        impl IntoIterator for Count {
            type IntoIter = Take<Repeat<()>>;
            type Item = ();

            fn into_iter(self) -> Self::IntoIter {
                iter::repeat(()).take(self.0)
            }
        }

        let input = [Some(()), Some(()), None];
        let nullable = input.into_iter().collect::<Nullable<Count>>();
        assert_eq!(nullable.validity_bitmap().buffer_ref(), &[0b00000011u8]);
        assert_eq!(nullable.into_iter().collect::<Vec<Option<()>>>(), input);
    }

    #[test]
    fn size_of() {
        assert_eq!(mem::size_of::<Nullable<()>>(), mem::size_of::<Bitmap>());
    }
}
