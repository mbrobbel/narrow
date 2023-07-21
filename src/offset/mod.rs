//! Offsets for variable-sized arrays.

// use std::{borrow::Borrow, marker::PhantomData};
use std::{num::TryFromIntError, ops::AddAssign};

use crate::{
    buffer::{Buffer, BufferType, VecBuffer},
    validity::Validity,
    FixedSize, Length,
};

use self::iter::ScanOffsetsExt;
// use crate::{
//     bitmap::{Bitmap, ValidityBitmap},
//     buffer::{Buffer, BufferAlloc, BufferExtend, BufferRef},
//     validity::Validity,
//     Length, Primitive,
// };

pub mod buffer;
mod iter;

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
    T,
    const NULLABLE: bool = false,
    OffsetItem: OffsetElement = i32,
    Buffer: BufferType = VecBuffer,
> where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
{
    pub data: T,
    pub offsets:
        <<Buffer as BufferType>::Buffer<OffsetItem> as Validity<NULLABLE>>::Storage<Buffer>,
}

impl<T, const NULLABLE: bool, OffsetItem: OffsetElement, Buffer: BufferType> Default
    for Offset<T, NULLABLE, OffsetItem, Buffer>
where
    T: Default,
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    <<Buffer as BufferType>::Buffer<OffsetItem> as Validity<NULLABLE>>::Storage<Buffer>: Default,
{
    fn default() -> Self {
        Self {
            data: Default::default(),
            offsets: Default::default(),
        }
    }
}

impl<T, U: IntoIterator, OffsetItem: OffsetElement, Buffer: BufferType> Extend<U>
    for Offset<T, false, OffsetItem, Buffer>
where
    T: Default + Extend<<U as IntoIterator>::Item>,
    U: Length,
    <<Buffer as BufferType>::Buffer<OffsetItem> as Validity<false>>::Storage<Buffer>:
        Extend<OffsetItem>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        let mut state = match self.offsets.as_slice().last() {
            Some(offset) => *offset,
            None => {
                self.offsets.extend(Some(OffsetItem::default()));
                OffsetItem::default()
            }
        };
        self.data.extend(iter.into_iter().flat_map(|item| {
            state += OffsetItem::try_from(item.len()).unwrap();
            self.offsets.extend(Some(state));
            item.into_iter()
        }));
    }
}

impl<T, U: IntoIterator, OffsetItem: OffsetElement, Buffer: BufferType> FromIterator<U>
    for Offset<T, false, OffsetItem, Buffer>
where
    T: Default + Extend<<U as IntoIterator>::Item>,
    U: Length,
    <<Buffer as BufferType>::Buffer<OffsetItem> as Validity<false>>::Storage<Buffer>:
        FromIterator<OffsetItem>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        let mut data = T::default();
        let offsets = iter
            .into_iter()
            .map(|item| {
                let len = item.len();
                data.extend(item.into_iter());
                len
            })
            .scan_offsets()
            .collect();
        Self { data, offsets }
    }
}

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
