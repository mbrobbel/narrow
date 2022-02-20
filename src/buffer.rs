use crate::Primitive;
use std::{
    borrow::{Borrow, BorrowMut},
    mem, slice,
};

/// A contiguous immutable memory buffer for data.
///
/// Read-only slice.
///
/// There is a blanket implementation, so that everything that implements
/// `Borrow<[T]> where T: Primitive` can be used as a `Buffer` in this
/// crate.
pub trait Buffer<T>
where
    T: Primitive,
    Self: Borrow<[T]>,
{
    fn as_bytes(&self) -> &[u8] {
        // Safety:
        // - The pointer returned by slice::as_ptr (via Borrow) points to
        //   slice::len() consecutive properly initialized values of type T,
        //   with size_of::<T> bytes per element.
        unsafe {
            slice::from_raw_parts(
                self.borrow().as_ptr() as *const u8,
                self.borrow().len() * mem::size_of::<T>(),
            )
        }
    }
}

impl<T, U> Buffer<T> for U
where
    T: Primitive,
    U: Borrow<[T]>,
{
}

/// A contiguous mutable memory buffer for data.
///
/// In-place mutation.
pub trait BufferMut<T>
where
    T: Primitive,
    Self: Buffer<T>,
    Self: BorrowMut<[T]>,
{
}

impl<T, U> BufferMut<T> for U
where
    T: Primitive,
    U: Buffer<T> + BorrowMut<[T]>,
{
}

/// An allocatable contiguous memory buffer for data.
///
/// Allocation.
pub trait BufferAlloc<T>
where
    T: Primitive,
    Self: Buffer<T>,
    Self: FromIterator<T>,
{
    // type Uninit<U>;

    // fn new_uninit(len: usize) -> &mut [MaybeUninit<T>]; //Self::Container<'_, MaybeUninit<T>>;
    // think about pre-allocating for specific nr of elements with MaybeUninit
}

impl<T> BufferAlloc<T> for Vec<T> where T: Primitive {}
impl<T> BufferAlloc<T> for Box<[T]> where T: Primitive {}

// impl<T> BufferAlloc<T> for Box<[T]>
// where
//     T: Primitive,
// {
//     type Uninit<U> = Box<[U]>;

//     fn new_uninit(len: usize) -> Self::Container<'_, MaybeUninit<T>> {
//         Box::new_uninit_slice(len)
//     }
// }

// impl<T> BufferAlloc<T> for Vec<T>
// where
//     T: Primitive,
// {
//     type Container<'a, U> = &'a mut [U];

//     fn new_uninit(len: usize) -> Self::Container<'_, MaybeUninit<T>> {
//         let mut vec = Vec::with_capacity(len);
//         vec.spare_capacity_mut()
//     }
//     fn
// }

// // remove this blanket impl if we want methods on BufferAlloc
// impl<T, U> BufferAlloc<T> for U
// where
//     T: Primitive,
//     U: FromIterator<T>,
// {
// }

/// An extendable contiguous memory buffer for data.
///
/// Growing and shrinking.
pub trait BufferExtend<T>
where
    T: Primitive,
    Self: Extend<T>,
{
}

impl<T, U> BufferExtend<T> for U
where
    T: Primitive,
    U: Extend<T>,
{
}

// /// A contiguous immutable memory buffer for data.
// ///
// /// Generic over the [Primitive] element type `T` stored in this buffer.
// // todo(mb): make generic over Allocator
// #[derive(Clone, Default, PartialEq, Eq, Hash)]
// pub struct Buffer<T>
// where
//     T: Primitive,
// {
//     inner: Vec<T>,
// }

// impl<T> Buffer<T>
// where
//     T: Primitive,
// {
//     /// Constructs a new, empty `Buffer<T>`.
//     ///
//     /// The buffer will not allocate until element are pushed onto it.
//     pub fn new() -> Self {
//         Self { inner: Vec::new() }
//     }

//     /// Constructs a new, empty `Buffer<T>` with the specified capacity.
//     ///
//     /// The buffer will be able to hold exactly `capacity` elements without reallocating.
//     pub fn with_capacity(capacity: usize) -> Self {
//         Self {
//             inner: Vec::with_capacity(capacity),
//         }
//     }
// }

// impl<T> AsRef<[u8]> for Buffer<T>
// where
//     T: Primitive,
// {
//     fn as_ref(&self) -> &[u8] {
//         // Safety:
//         // - Length (number of elements) is an invariant of an immutable buffer.
//         unsafe {
//             slice::from_raw_parts(
//                 self.inner.as_ptr() as *const u8,
//                 self.len() * mem::size_of::<T>(),
//             )
//         }
//     }
// }

// impl<T> Debug for Buffer<T>
// where
//     T: Primitive,
// {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result {
//         f.debug_struct(&format!("Buffer<{}>", any::type_name::<T>()))
//             .field("values", &self.inner)
//             .finish()
//     }
// }

// impl<T> Deref for Buffer<T>
// where
//     T: Primitive,
// {
//     type Target = Vec<T>;

//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

// impl<T> DerefMut for Buffer<T>
// where
//     T: Primitive,
// {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner
//     }
// }

// impl<T> Length for Buffer<T>
// where
//     T: Primitive,
// {
//     fn len(&self) -> usize {
//         self.inner.len()
//     }
// }

// // impl<T> Extend<T> for Buffer<T>
// // where
// //     T: Primitive,
// // {
// //     fn extend<I>(&mut self, iter: I)
// //     where
// //         I: IntoIterator<Item = T>,
// //     {
// //         self.inner.extend(iter)
// //     }
// // }

// // impl<'a, T> Extend<&'a T> for Buffer<T>
// // where
// //     T: Primitive + 'a,
// // {
// //     fn extend<I>(&mut self, iter: I)
// //     where
// //         I: IntoIterator<Item = &'a T>,
// //     {
// //         self.inner.extend(iter)
// //     }
// // }

// impl<T> From<&[T]> for Buffer<T>
// where
//     T: Primitive,
// {
//     fn from(slice: &[T]) -> Self {
//         Self {
//             inner: slice.into(),
//         }
//     }
// }

// // impl<T, const N: usize> From<[T; N]> for Buffer<T>
// // where
// //     T: Primitive,
// // {
// //     fn from(array: [T; N]) -> Self {
// //         Self {
// //             inner: array.into(),
// //         }
// //     }
// // }

// // impl<T> From<Box<[T]>> for Buffer<T>
// // where
// //     T: Primitive,
// // {
// //     fn from(boxed_slice: Box<[T]>) -> Self {
// //         Self {
// //             inner: boxed_slice.into(),
// //         }
// //     }
// // }

// // impl<'a, T> From<Cow<'a, [T]>> for Buffer<T>
// // where
// //     T: Primitive,
// // {
// //     fn from(cow: Cow<'a, [T]>) -> Self {
// //         Self { inner: cow.into() }
// //     }
// // }

// impl<T> From<Vec<T>> for Buffer<T>
// where
//     T: Primitive,
// {
//     fn from(vec: Vec<T>) -> Self {
//         Self { inner: vec }
//     }
// }

// impl<T> From<Buffer<T>> for Vec<T>
// where
//     T: Primitive,
// {
//     fn from(buffer: Buffer<T>) -> Self {
//         buffer.inner
//     }
// }

// impl<'a, T> FromIterator<&'a T> for Buffer<T>
// where
//     T: Primitive + 'a,
// {
//     fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Self {
//         Self {
//             inner: iter.into_iter().copied().collect(),
//         }
//     }
// }

// impl<T> FromIterator<T> for Buffer<T>
// where
//     T: Primitive,
// {
//     fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
//         Self {
//             inner: iter.into_iter().collect(),
//         }
//     }
// }

// impl<T, I> Index<I> for Buffer<T>
// where
//     T: Primitive,
//     I: SliceIndex<[T]>,
// {
//     type Output = I::Output;

//     fn index(&self, index: I) -> &Self::Output {
//         self.inner.index(index)
//     }
// }

// impl<T, I> IndexMut<I> for Buffer<T>
// where
//     T: Primitive,
//     I: SliceIndex<[T]>,
// {
//     fn index_mut(&mut self, index: I) -> &mut Self::Output {
//         self.inner.index_mut(index)
//     }
// }

// impl<'a, T> IntoIterator for &'a Buffer<T>
// where
//     T: Primitive,
// {
//     type Item = &'a T;
//     type IntoIter = <&'a Vec<T> as IntoIterator>::IntoIter;

//     fn into_iter(self) -> Self::IntoIter {
//         self.inner.iter()
//     }
// }

// impl<T> IntoIterator for Buffer<T>
// where
//     T: Primitive,
// {
//     type Item = T;
//     type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

//     fn into_iter(self) -> Self::IntoIter {
//         self.inner.into_iter()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn as_ref_u8() {
//         let vec = vec![42u32, u32::MAX, 0xc0fefe];
//         let buffer = vec.into_iter().collect::<Buffer<_>>();
//         assert_eq!(
//             buffer.as_ref(),
//             &[42u8, 0, 0, 0, 255, 255, 255, 255, 254, 254, 192, 0]
//         );
//     }

//     #[test]
//     fn from_iter() {
//         let vec = vec![1u32, 2, 3, 4];
//         let buffer = vec.iter().collect::<Buffer<_>>();
//         assert_eq!(buffer.len(), 4);
//         assert_eq!(&vec[..], &buffer[..]);
//     }

//     #[test]
//     fn into_iter() {
//         let vec = vec![1u32, 2, 3, 4];
//         assert_eq!(
//             vec,
//             vec.iter()
//                 .collect::<Buffer<_>>()
//                 .into_iter()
//                 .collect::<Vec<_>>()
//         );
//     }
// }

// // }
