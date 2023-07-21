//! Offsets for variable sized arrays.

// use std::{borrow::Borrow, marker::PhantomData};
use std::{num::TryFromIntError, ops::AddAssign};

use crate::Primitive;

// use self::iter::ScanOffsetsExt;
// use crate::{
//     bitmap::{Bitmap, ValidityBitmap},
//     buffer::{Buffer, BufferAlloc, BufferExtend, BufferRef},
//     validity::Validity,
//     Length, Primitive,
// };

// pub mod buffer;
// mod iter;

// pub mod buffer;

/// Types representing offset values.
///
/// Values with these types can be used to represent offset values.
///
/// This trait is sealed to prevent downstream implementations.
pub trait OffsetElement:
    Primitive
    + AddAssign
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

// pub struct Offset<const NULLABLE: bool, T, OffsetElement, Buffer>
// where
//     OffsetElement: self::OffsetElement,
//     OffsetBuffer: Buffer<OffsetElement> + Validity<NULLABLE>,
//     BitmapBuffer: Buffer<u8>,
// {
//     data: T,
//     offsets: <OffsetBuffer as Validity<NULLABLE>>::Storage<BitmapBuffer>,
//     _element_ty: PhantomData<fn() -> OffsetElement>,
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
