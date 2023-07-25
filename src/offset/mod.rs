//! Offsets for variable-sized arrays.

// use self::iter::ScanOffsetsExt;
use crate::{
    buffer::{Buffer, BufferType, VecBuffer},
    validity::Validity,
    FixedSize, Length,
};
use std::{num::TryFromIntError, ops::AddAssign};

// mod iter;

/// Types representing offset values.
///
/// Values with these types can be used to represent offset values.
///
/// This trait is sealed to prevent downstream implementations.
pub trait OffsetElement:
    FixedSize
    + AddAssign
    + Default
    + TryFrom<usize, Error = TryFromIntError>
    + TryInto<usize, Error = TryFromIntError>
    + sealed::Sealed
{
}

mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for T where T: super::OffsetElement {}
}

impl OffsetElement for i32 {}
impl OffsetElement for i64 {}

pub struct Offset<
    T: Validity<NULLABLE>,
    const NULLABLE: bool = false,
    OffsetItem: OffsetElement = i32,
    Buffer: BufferType = VecBuffer,
> {
    // A downside of this approach is getting default values in data for nulls?
    // TODO(mbrobbel): maybe go back to wrapping the offsets buffer with validity - however that has the downside of length difference
    pub data: <T as Validity<NULLABLE>>::Storage<Buffer>,
    pub offsets: <Buffer as BufferType>::Buffer<OffsetItem>,
}

impl<
        T: Validity<NULLABLE>,
        const NULLABLE: bool,
        OffsetItem: OffsetElement,
        Buffer: BufferType,
    > Default for Offset<T, NULLABLE, OffsetItem, Buffer>
where
    <T as Validity<NULLABLE>>::Storage<Buffer>: Default,
    <Buffer as BufferType>::Buffer<OffsetItem>: Default + Extend<OffsetItem>,
{
    fn default() -> Self {
        let mut offsets = <Buffer as BufferType>::Buffer::<OffsetItem>::default();
        offsets.extend(std::iter::once(OffsetItem::default()));
        Self {
            data: Default::default(),
            offsets,
        }
    }
}

impl<
        T: Validity<NULLABLE>,
        U: Length,
        const NULLABLE: bool,
        OffsetItem: OffsetElement,
        Buffer: BufferType,
    > Extend<U> for Offset<T, NULLABLE, OffsetItem, Buffer>
where
    <T as Validity<NULLABLE>>::Storage<Buffer>: Extend<U>,
    <Buffer as BufferType>::Buffer<OffsetItem>: Extend<OffsetItem>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        let mut state = self.offsets.as_slice().last().copied().unwrap();
        self.data.extend(iter.into_iter().inspect(|item| {
            state += OffsetItem::try_from(item.len()).unwrap();
            self.offsets.extend(std::iter::once(state));
        }));
    }
}

impl<T, U: IntoIterator + Length, OffsetItem: OffsetElement, Buffer: BufferType> FromIterator<U>
    for Offset<T, false, OffsetItem, Buffer>
where
    Self: Default,
    <T as Validity<false>>::Storage<Buffer>: FromIterator<<U as IntoIterator>::Item>,
    <Buffer as BufferType>::Buffer<OffsetItem>: Extend<OffsetItem>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        let mut offset = Self::default();
        let mut state = offset.offsets.as_slice().last().copied().unwrap();
        offset.data = iter
            .into_iter()
            .inspect(|item| {
                state += OffsetItem::try_from(item.len()).unwrap();
                offset.offsets.extend(std::iter::once(state));
            })
            .flat_map(IntoIterator::into_iter)
            .collect();
        offset
    }
}

impl<T, U: IntoIterator + Length, OffsetItem: OffsetElement, Buffer: BufferType>
    FromIterator<Option<U>> for Offset<T, true, OffsetItem, Buffer>
where
    Self: Default,
    <T as Validity<true>>::Storage<Buffer>: FromIterator<Option<<U as IntoIterator>::Item>>,
    <Buffer as BufferType>::Buffer<OffsetItem>: Extend<OffsetItem>,
{
    fn from_iter<I: IntoIterator<Item = Option<U>>>(iter: I) -> Self {
        enum Either<L, R> {
            Left(L),
            Right(R),
        }
        impl<L, R> Iterator for Either<L, R>
        where
            L: Iterator,
            R: Iterator<Item = L::Item>,
        {
            type Item = L::Item;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Either::Left(ref mut left) => left.next(),
                    Either::Right(ref mut right) => right.next(),
                }
            }
        }

        let mut offset = Self::default();
        let mut state = offset.offsets.as_slice().last().copied().unwrap();
        offset.data = iter
            .into_iter()
            .inspect(|item| {
                state += OffsetItem::try_from(item.len()).unwrap();
                offset.offsets.extend(std::iter::once(state));
            })
            .flat_map(|opt| match opt {
                Some(item) => Either::Left(item.into_iter().map(Option::Some)),
                None => Either::Right(std::iter::once(Option::<<U as IntoIterator>::Item>::None)),
            })
            .collect();
        offset
    }
}

impl<
        T: Validity<NULLABLE>,
        const NULLABLE: bool,
        OffsetItem: OffsetElement,
        Buffer: BufferType,
    > Length for Offset<T, NULLABLE, OffsetItem, Buffer>
where
    <T as Validity<NULLABLE>>::Storage<Buffer>: Length,
{
    fn len(&self) -> usize {
        self.offsets.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bitmap::{BitmapRef, ValidityBitmap},
        buffer::BufferRef,
    };

    #[test]
    fn default() {
        let offset = Offset::<()>::default();
        assert_eq!(offset.offsets.as_slice(), &[0]);

        let offset = Offset::<(), true>::default();
        assert_eq!(offset.offsets.as_slice(), &[0]);
        assert_eq!(offset.len(), 0);
    }

    #[test]
    fn extend() {
        let mut offset = Offset::<Vec<Vec<u8>>>::default();
        offset.extend(std::iter::once(vec![1, 2, 3, 4]));
        assert_eq!(offset.len(), 1);
        assert_eq!(offset.offsets.as_slice(), &[0, 4]);
        offset.extend(std::iter::once(vec![5]));
        assert_eq!(offset.offsets.as_slice(), &[0, 4, 5]);
        assert_eq!(offset.len(), 2);

        let mut offset = Offset::<Vec<Vec<u8>>, true>::default();
        offset.extend(vec![Some(vec![1, 2, 3, 4]), None, None]);
        assert_eq!(offset.offsets.as_slice(), &[0, 4, 4, 4]);
        assert_eq!(offset.len(), 3);

        let mut offset = Offset::<Vec<String>, true>::default();
        offset.extend(vec![
            Some("asf".to_string()),
            None,
            Some("asdf".to_string()),
        ]);
        assert_eq!(offset.data.bitmap_ref().valid_count(), 2);
    }

    #[test]
    fn from_iter() {
        let input = vec![vec![1, 2, 3, 4], vec![5, 6], vec![7, 8, 9]];
        let offset = input.into_iter().collect::<Offset<Vec<u8>>>();
        assert_eq!(offset.len(), 3);
        assert_eq!(offset.offsets.as_slice(), &[0, 4, 6, 9]);
        assert_eq!(offset.data, &[1u8, 2, 3, 4, 5, 6, 7, 8, 9]);

        let input = vec![Some(["a".to_string()]), None, Some(["b".to_string()]), None];
        let offset = input.into_iter().collect::<Offset<String, true>>();
        assert_eq!(offset.len(), 4);
        assert_eq!(offset.offsets.as_slice(), &[0, 1, 1, 2, 2]);
        assert_eq!(
            offset.data.bitmap_ref().buffer_ref().as_slice(),
            &[0b0000_00101]
        );
    }
}

// impl<T, U, OffsetItem: OffsetElement, Buffer: BufferType> Extend<U>
//     for Offset<T, false, OffsetItem, Buffer>
// where
//     T: Default + Extend<<U as IntoIterator>::Item>,
//     U: IntoIterator + Length,
//     <<Buffer as BufferType>::Buffer<OffsetItem> as Validity<false>>::Storage<Buffer>:
//         Extend<OffsetItem>,
// {
//     fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
//         // Get the current offset or insert first value if empty.
//         let mut state = self.offsets.as_slice().last().copied().unwrap_or_else(|| {
//             self.offsets.extend(Some(OffsetItem::default()));
//             OffsetItem::default()
//         });

//         // Extend data with items from the iterators.
//         self.data.extend(iter.into_iter().flat_map(|item| {
//             state += OffsetItem::try_from(item.len()).unwrap();
//             self.offsets.extend(Some(state));
//             item.into_iter()
//         }));
//     }
// }

// impl<T, U, OffsetItem: OffsetElement, Buffer: BufferType> Extend<Option<U>>
//     for Offset<T, true, OffsetItem, Buffer>
// where
//     T: Default + Extend<<U as IntoIterator>::Item>,
//     U: IntoIterator + Length,
//     <<Buffer as BufferType>::Buffer<OffsetItem> as Validity<true>>::Storage<Buffer>:
//         Extend<Option<OffsetItem>>,
// {
//     fn extend<I: IntoIterator<Item = Option<U>>>(&mut self, _iter: I) {
//         // If the offsets buffer is still empty set the first value in the data buffer,
//         // but don't extend the bitmap.
//         // Get the current offset or insert first value if empty.
//         // let mut state = self
//         //     .offsets
//         //     .as_ref()
//         //     .as_slice()
//         //     .last()
//         //     .copied()
//         //     .unwrap_or_else(|| {
//         //         self.offsets.extend(Some(None));
//         //         OffsetItem::default()
//         //     });
//         // self.data.extend(iter.into_iter().flat_map(|opt| match opt {
//         //     Some(item) => {
//         //         state += OffsetItem::try_from(item.len()).unwrap();
//         //         self.offsets.extend(Some(state));
//         //         opt.into_iter()
//         //     }
//         //     None => {
//         //         self.offsets.extend(Some(state));
//         //         opt.into_iter()
//         //     }
//         // }));
//         todo!()
//     }
// }

// impl<T, U: IntoIterator + Length, OffsetItem: OffsetElement, Buffer: BufferType> FromIterator<U>
//     for Offset<T, false, OffsetItem, Buffer>
// where
//     T: Default + Extend<<U as IntoIterator>::Item>,
//     <<Buffer as BufferType>::Buffer<OffsetItem> as Validity<false>>::Storage<Buffer>:
//         FromIterator<OffsetItem>,
// {
//     fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
//         let mut data = T::default();
//         let offsets = iter
//             .into_iter()
//             .map(|item| {
//                 let len = item.len();
//                 data.extend(item.into_iter());
//                 len
//             })
//             .scan_offsets()
//             .collect();
//         Self { data, offsets }
//     }
// }

// impl<T, U: IntoIterator + Length, OffsetItem: OffsetElement, Buffer: BufferType>
//     FromIterator<Option<U>> for Offset<T, true, OffsetItem, Buffer>
// where
//     T: Default + Extend<<U as IntoIterator>::Item>,
//     <<Buffer as BufferType>::Buffer<OffsetItem> as Validity<true>>::Storage<Buffer>:
//         FromIterator<Option<OffsetItem>>,
// {
//     fn from_iter<I: IntoIterator<Item = Option<U>>>(iter: I) -> Self {
//         let mut data = T::default();
//         let mut state = OffsetItem::default();
//         let offsets = iter
//             .into_iter()
//             .flat_map(|opt| match opt {
//                 Some(item) => {
//                     state += OffsetItem::try_from(item.len()).unwrap();
//                     data.extend(item);
//                     Some(Some(state))
//                     // (true, std::iter::once(state))
//                 }
//                 None => {
//                     // (false, std::iter::once(state))
//                     Some(None)
//                 }
//             })
//             .collect();
//         Self { data, offsets }
//     }
// }

// impl<T, Data, OffsetElement, OffsetBuffer, BitmapBuffer> FromIterator<Option<T>>
//     for Offset<true, Data, OffsetElement, OffsetBuffer, BitmapBuffer>
// where
//     T: IntoIterator + Length,
//     Data: Default + Extend<<T as IntoIterator>::Item>,
//     OffsetElement: self::OffsetElement,
//     OffsetBuffer: Default + BufferExtend<OffsetElement>,
//     BitmapBuffer: BufferAlloc<u8>,
// {
//     fn from_iter<I: IntoIterator<Item = Option<T>>>(iter: I) -> Self {
//         // TODO(mbrobbel): optimize pre-alloc
//         let mut data = Data::default();
//         let mut state = OffsetElement::default();
//         let offsets = iter
//             .into_iter()
//             .map(|opt| match opt {
//                 Some(item) => {
//                     state += OffsetElement::try_from(item.len()).unwrap();
//                     data.extend(item);
//                     (true, std::iter::once(state))
//                 }
//                 None => (false, std::iter::once(state)),
//             })
//             .collect();
//         Self {
//             data,
//             offsets,
//             _element_ty: PhantomData,
//         }
//     }
// }

// impl<const NULLABLE: bool, Data, OffsetElement, OffsetBuffer, BitmapBuffer> BufferRef
//     for Offset<NULLABLE, Data, OffsetElement, OffsetBuffer, BitmapBuffer>
// where
//     Data: BufferRef,
//     OffsetElement: self::OffsetElement,
//     OffsetBuffer: Buffer<OffsetElement> + Validity<NULLABLE>,
//     BitmapBuffer: Buffer<u8>,
// {
//     type Buffer = <Data as BufferRef>::Buffer;
//     type Element = <Data as BufferRef>::Element;

//     fn buffer_ref(&self) -> &Self::Buffer {
//         self.data.buffer_ref()
//     }
// }

// impl<Data, OffsetElement, OffsetBuffer, BitmapBuffer> ValidityBitmap
//     for Offset<true, Data, OffsetElement, OffsetBuffer, BitmapBuffer>
// where
//     BitmapBuffer: Buffer<u8>,
//     OffsetElement: self::OffsetElement,
//     OffsetBuffer: Buffer<OffsetElement>,
// {
//     type Buffer = BitmapBuffer;

//     #[inline]
//     fn validity_bitmap(&self) -> &Bitmap<Self::Buffer> {
//         self.offsets.validity_bitmap()
//     }

//     #[inline]
//     fn validity_bitmap_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
//         self.offsets.validity_bitmap_mut()
//     }
// }

// impl<const NULLABLE: bool, Data, OffsetElement, OffsetBuffer, BitmapBuffer> Length
//     for Offset<NULLABLE, Data, OffsetElement, OffsetBuffer, BitmapBuffer>
// where
//     OffsetElement: self::OffsetElement,
//     OffsetBuffer: Buffer<OffsetElement> + Validity<NULLABLE>,
//     BitmapBuffer: Buffer<u8>,
//     <OffsetBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer>: BufferRef,
// {
//     fn len(&self) -> usize {
//         // The offsets buffer stores an additional value
//         self.offsets.buffer_ref().borrow().len() - 1
//     }
// }

// impl<T, Data, OffsetElement, OffsetBuffer, BitmapBuffer> FromIterator<T>
//     for Offset<false, Data, OffsetElement, OffsetBuffer, BitmapBuffer>
// where
//     T: IntoIterator + Length,
//     Data: Default + Extend<<T as IntoIterator>::Item>,
//     OffsetElement: self::OffsetElement,
//     BitmapBuffer: Buffer<u8>,
//     OffsetBuffer: Buffer<OffsetElement> + FromIterator<OffsetElement>,
// {
//     fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
//         let mut data = Data::default();
//         let offsets = iter
//             .into_iter()
//             .map(|item| {
//                 let len = item.len();
//                 data.extend(item.into_iter());
//                 len
//             })
//             .scan_offsets()
//             .collect();
//         Self {
//             data,
//             offsets,
//             _element_ty: PhantomData,
//         }
//     }
// }

// impl<T, Data, OffsetElement, OffsetBuffer, BitmapBuffer> FromIterator<Option<T>>
//     for Offset<true, Data, OffsetElement, OffsetBuffer, BitmapBuffer>
// where
//     T: IntoIterator + Length,
//     Data: Default + Extend<<T as IntoIterator>::Item>,
//     OffsetElement: self::OffsetElement,
//     OffsetBuffer: Default + BufferExtend<OffsetElement>,
//     BitmapBuffer: BufferAlloc<u8>,
// {
//     fn from_iter<I: IntoIterator<Item = Option<T>>>(iter: I) -> Self {
//         // TODO(mbrobbel): optimize pre-alloc
//         let mut data = Data::default();
//         let mut state = OffsetElement::default();
//         let offsets = iter
//             .into_iter()
//             .map(|opt| match opt {
//                 Some(item) => {
//                     state += OffsetElement::try_from(item.len()).unwrap();
//                     data.extend(item);
//                     (true, std::iter::once(state))
//                 }
//                 None => (false, std::iter::once(state)),
//             })
//             .collect();
//         Self {
//             data,
//             offsets,
//             _element_ty: PhantomData,
//         }
//     }
// }

// impl<const NULLABLE: bool, Data, OffsetElement, OffsetBuffer, BitmapBuffer>
//     buffer::OffsetBuffer<OffsetElement>
//     for Offset<NULLABLE, Data, OffsetElement, OffsetBuffer, BitmapBuffer>
// where
//     OffsetElement: self::OffsetElement,
//     OffsetBuffer: Buffer<OffsetElement> + Validity<NULLABLE>,
//     BitmapBuffer: Buffer<u8>,
//     <OffsetBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer>: BufferRef,
//     <<OffsetBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer> as BufferRef>::Buffer:
//         Buffer<OffsetElement>,
// {
//     type Buffer =
//         <<OffsetBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer> as BufferRef>::Buffer;

//     fn offset_buffer(&self) -> &Self::Buffer {
//         self.offsets.buffer_ref()
//     }
// }
