use super::Array;
use crate::{
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{BufferType, VecBuffer},
    validity::Validity,
    FixedSize, Length,
};

/// Array with primitive values.
pub struct FixedSizePrimitiveArray<
    T: FixedSize,
    const NULLABLE: bool = false,
    Buffer: BufferType = VecBuffer,
>(pub <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>)
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>;

macro_rules! type_def {
    ($ident:ident, $ty:ty) => {
        #[doc = "Array with ["]
        #[doc = stringify!($ty)]
        #[doc = "] values."]
        pub type $ident<const NULLABLE: bool = false, Buffer = VecBuffer> =
            FixedSizePrimitiveArray<$ty, NULLABLE, Buffer>;
    };
}

type_def!(Int8Array, i8);
type_def!(Int16Array, i16);
type_def!(Int32Array, i32);
type_def!(Int64Array, i64);
type_def!(Uint8Array, u8);
type_def!(Uint16Array, u16);
type_def!(Uint32Array, u32);
type_def!(Uint64Array, u64);

type_def!(IsizeArray, isize);
type_def!(UsizeArray, usize);

type_def!(Float32Array, f32);
type_def!(Float64Array, f64);

impl<T: FixedSize, const NULLABLE: bool, Buffer: BufferType> Array
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
{
}

impl<T: FixedSize, const NULLABLE: bool, Buffer: BufferType> Default
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: FixedSize, U, const NULLABLE: bool, Buffer: BufferType> Extend<U>
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: Extend<U>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl<T: FixedSize, const NULLABLE: bool, U, Buffer: BufferType> FromIterator<U>
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: FromIterator<U>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'a, T: FixedSize, const NULLABLE: bool, Buffer: BufferType> IntoIterator
    for &'a FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    &'a <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: IntoIterator,
{
    type Item = <&'a <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::Item;
    type IntoIter = <&'a <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: FixedSize, const NULLABLE: bool, Buffer: BufferType> IntoIterator
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: IntoIterator,
{
    type Item = <<<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::Item;
    type IntoIter = <<<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: FixedSize, const NULLABLE: bool, Buffer: BufferType> Length
    for FixedSizePrimitiveArray<T, NULLABLE, Buffer>
where
    <Buffer as BufferType>::Buffer<T>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<T> as Validity<NULLABLE>>::Storage<Buffer>: Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: FixedSize, Buffer: BufferType> BitmapRef for FixedSizePrimitiveArray<T, true, Buffer> {
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<T: FixedSize, Buffer: BufferType> BitmapRefMut for FixedSizePrimitiveArray<T, true, Buffer> {
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<T: FixedSize, Buffer: BufferType> ValidityBitmap for FixedSizePrimitiveArray<T, true, Buffer> {}

#[cfg(feature = "arrow-array")]
mod arrow {
    use super::FixedSizePrimitiveArray;
    use crate::{
        bitmap::Bitmap,
        buffer::{ArrowBuffer, BufferType},
        FixedSize, Length,
    };
    use arrow_array::{types::ArrowPrimitiveType, PrimitiveArray};
    use arrow_buffer::{BooleanBuffer, NullBuffer, ScalarBuffer};

    impl<T: FixedSize, U: ArrowPrimitiveType<Native = T>, Buffer: BufferType>
        From<FixedSizePrimitiveArray<T, false, Buffer>> for PrimitiveArray<U>
    where
        <Buffer as BufferType>::Buffer<T>: Length + Into<<ArrowBuffer as BufferType>::Buffer<T>>,
    {
        fn from(value: FixedSizePrimitiveArray<T, false, Buffer>) -> Self {
            let len = value.len();
            Self::new(ScalarBuffer::new(value.0.into().finish(), 0, len), None)
        }
    }

    impl<T: FixedSize, U: ArrowPrimitiveType<Native = T>, Buffer: BufferType>
        From<FixedSizePrimitiveArray<T, true, Buffer>> for PrimitiveArray<U>
    where
        <Buffer as BufferType>::Buffer<T>: Length + Into<<ArrowBuffer as BufferType>::Buffer<T>>,
        Bitmap<Buffer>: Into<BooleanBuffer>,
    {
        fn from(value: FixedSizePrimitiveArray<T, true, Buffer>) -> Self {
            let len = value.len();
            Self::new(
                ScalarBuffer::new(value.0.data.into().finish(), 0, len),
                Some(NullBuffer::new(value.0.validity.into())),
            )
        }
    }

    #[cfg(test)]
    mod test {

        #[test]
        #[cfg(feature = "arrow-array")]
        fn arrow_array() {
            use crate::{array::Int8Array, bitmap::ValidityBitmap, buffer::ArrowBuffer};
            use arrow_array::{types::Int8Type, Array, PrimitiveArray};

            let input = [1, 2, 3, 4];
            let array = input.into_iter().collect::<Int8Array<false, ArrowBuffer>>();
            let array = PrimitiveArray::<Int8Type>::from(array);
            assert_eq!(array.len(), 4);

            // let input = [1, 2, 3, 4];
            // let array = input.into_iter().collect::<Int8Array<false>>();
            // let array = PrimitiveArray::<Int8Type>::from(array);
            // assert_eq!(array.len(), 4);

            let input = [Some(1), None, Some(3), Some(4)];
            let array = input.into_iter().collect::<Int8Array<true, ArrowBuffer>>();
            assert_eq!(array.null_count(), 1);
            let array = PrimitiveArray::<Int8Type>::from(array);
            assert_eq!(array.len(), 4);
            assert_eq!(array.null_count(), 1);
        }

        #[test]
        fn convert() {}
    }
}

pub use arrow::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bitmap::Bitmap,
        buffer::{Buffer, BufferRef},
    };
    use std::mem;

    #[test]
    fn from_iter() {
        let input = [1u8, 2, 3, 4];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array.0.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(array.0.as_slice(), array.0.as_bytes());

        let input = [[1u8, 2], [3, 4]];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array.0.as_slice(), &[[1, 2], [3, 4]]);
        assert_eq!(<_ as Buffer<u8>>::as_bytes(&array.0), &[1, 2, 3, 4]);

        let input = [Some(1u64), None, Some(3), Some(4)];
        let array = input.iter().collect::<FixedSizePrimitiveArray<_, true>>();
        assert_eq!(array.0.buffer_ref().as_slice(), &[1, u64::default(), 3, 4]);
        assert_eq!(array.is_valid(0), Some(true));
        assert_eq!(array.is_null(1), Some(true));
        assert_eq!(array.is_valid(2), Some(true));
        assert_eq!(array.is_valid(3), Some(true));
        assert_eq!(array.is_valid(4), None);
    }

    #[test]
    fn into_iter() {
        let input = [1u8, 2, 3, 4];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), input);

        let input = [[1u8, 2], [3, 4]];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), input);

        let input = [Some(1u64), None, Some(3), Some(4)];
        let array = input.iter().collect::<FixedSizePrimitiveArray<_, true>>();
        assert_eq!(array.into_iter().collect::<Vec<_>>(), input);
    }

    #[test]
    fn length() {
        let input = [1u8, 2, 3, 4];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array.len(), input.as_slice().len());

        let input = [[1u8, 2], [3, 4]];
        let array = input.into_iter().collect::<FixedSizePrimitiveArray<_>>();
        assert_eq!(array.len(), input.as_slice().len());

        let input = [Some(1u64), None, Some(3), Some(4)];
        let array = input.iter().collect::<FixedSizePrimitiveArray<_, true>>();
        assert_eq!(array.len(), input.len());
    }

    #[test]
    fn size_of() {
        assert_eq!(mem::size_of::<Int8Array>(), mem::size_of::<Vec<i8>>());
        assert_eq!(
            std::mem::size_of::<Int8Array<true>>(),
            mem::size_of::<Int8Array>() + mem::size_of::<Bitmap>()
        );
    }

    #[test]
    #[cfg(feature = "arrow-buffer")]
    fn arrow_buffer() {
        use crate::buffer::ArrowBuffer;

        let input = [1, 2, 3, 4];
        let mut array = input.into_iter().collect::<Int8Array<false, ArrowBuffer>>();
        assert_eq!(array.len(), 4);
        // Use arrow_buffer
        array.0.append_n(5, 5);
        assert_eq!(array.0.as_slice(), &[1, 2, 3, 4, 5, 5, 5, 5, 5]);

        let input = [Some(1), None, Some(3), Some(4)];
        let array = input.into_iter().collect::<Int8Array<true, ArrowBuffer>>();
        assert_eq!(array.len(), 4);
    }
}
