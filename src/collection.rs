//!

use std::{
    borrow::{Borrow, BorrowMut},
    iter::{self, Copied, Map, Repeat, Zip},
    marker::PhantomData,
    mem,
    ops::Range,
    rc::Rc,
    slice,
    sync::Arc,
};

use crate::{FixedSize, Length};

///
pub trait Collection: Length {
    ///
    type Item;
    ///
    type RefItem<'a>
    where
        Self: 'a;

    ///
    type Iter<'a>: Iterator<Item = Self::RefItem<'a>>
    where
        Self: 'a;
    ///
    fn iter(&self) -> Self::Iter<'_>;

    ///
    type IntoIter: Iterator<Item = Self::Item>;
    ///
    fn into_iter(self) -> Self::IntoIter;
}

///
pub trait CollectionMut {}
///
pub trait CollectionAlloc: Default {}

///
pub trait BufferType {
    ///
    type Buffer<T: FixedSize>: Buffer<T>;
}
// ///
// pub trait BufferAllocType: BufferType {
//     ///
//     type BufferAlloc<T: FixedSize>: BufferAlloc<T>;
// }

///
pub trait Buffer<T: FixedSize>: Borrow<[T]> + Collection<Item = T> {
    ///
    fn as_slice(&self) -> &[T] {
        self.borrow()
    }

    /// Returns the contents of the entire buffer as a byte slice.
    fn as_bytes(&self) -> &[u8] {
        // Safety:
        // - The pointer returned by slice::as_ptr (via Borrow) points to slice::len()
        //   consecutive properly initialized values of type T, with size_of::<T> bytes
        //   per element.
        unsafe {
            slice::from_raw_parts(
                self.as_slice().as_ptr().cast(),
                mem::size_of_val(self.as_slice()),
            )
        }
    }

    //     ///
    //     fn iter(&self) -> slice::Iter<'_, T> {
    //         self.borrow().iter()
    //     }

    //     ///
    //     type BufferIntoIter: Iterator<Item = T>;
    //     ///
    //     fn into_iter(self) -> Self::BufferIntoIter;
}

impl<T: FixedSize, U: Borrow<[T]> + Collection<Item = T> + Length> Buffer<T> for U {}

///
pub trait BufferMut<T: FixedSize>: Buffer<T> + BorrowMut<[T]> {
    ///
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.borrow_mut()
    }

    ///
    fn as_mut_bytes(&mut self) -> &mut [u8] {
        // Safety:
        // - The pointer returned by slice::as_mut_ptr (via BorrowMut) points to slice::len()
        //   consecutive properly initialized values of type T, with size_of::<T> bytes
        //   per element.
        unsafe {
            slice::from_raw_parts_mut(
                self.as_mut_slice().as_mut_ptr().cast(),
                mem::size_of_val(self.as_slice()),
            )
        }
    }

    ///
    fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.borrow_mut().iter_mut()
    }
}

impl<T: FixedSize, U: Buffer<T> + BorrowMut<[T]>> BufferMut<T> for U {}

///
pub trait BufferAlloc<T: FixedSize>: Buffer<T> + Default + Extend<T> {
    ///
    fn with_capacity(capacity: usize) -> Self;
    ///
    fn reserve(&mut self, additional: usize);
}

impl<T: FixedSize> BufferAlloc<T> for Vec<T> {
    fn with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity)
    }

    fn reserve(&mut self, additional: usize) {
        Vec::reserve(self, additional);
    }
}

impl<T: FixedSize, const N: usize> Collection for [T; N] {
    type Item = T;

    type RefItem<'a>
        = &'a T
    where
        Self: 'a;

    type Iter<'a>
        = slice::Iter<'a, T>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }

    type IntoIter = <[T; N] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <[T; N] as IntoIterator>::into_iter(self)
    }
}
///
#[derive(Clone, Copy)]
pub struct ArrayBuffer<const N: usize>;
impl<const N: usize> BufferType for ArrayBuffer<N> {
    type Buffer<T: FixedSize> = [T; N];
}

impl<T: FixedSize> Collection for Vec<T> {
    type Item = T;

    type RefItem<'a>
        = &'a T
    where
        Self: 'a;

    type Iter<'a>
        = slice::Iter<'a, T>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        <&Vec<T> as IntoIterator>::into_iter(self)
    }

    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <Vec<T> as IntoIterator>::into_iter(self)
    }
}
///
#[derive(Clone, Copy)]
pub struct VecBuffer;
impl BufferType for VecBuffer {
    type Buffer<T: FixedSize> = Vec<T>;
}

impl<T: FixedSize> Collection for Box<[T]> {
    type Item = T;

    type RefItem<'a>
        = &'a T
    where
        Self: 'a;

    type Iter<'a>
        = slice::Iter<'a, T>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        <&Box<[T]> as IntoIterator>::into_iter(self)
    }

    type IntoIter = <Box<[T]> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <Box<[T]> as IntoIterator>::into_iter(self)
    }
}
///
#[derive(Clone, Copy)]
pub struct BoxBuffer;
impl BufferType for BoxBuffer {
    type Buffer<T: FixedSize> = Box<[T]>;
}

impl<T: FixedSize> Collection for Arc<[T]> {
    type Item = T;

    type RefItem<'a>
        = &'a T
    where
        Self: 'a;

    type Iter<'a>
        = slice::Iter<'a, T>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }

    type IntoIter = Map<Zip<Repeat<Arc<[T]>>, Range<usize>>, fn((Arc<[T]>, usize)) -> T>;

    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        iter::repeat(self).zip(0..len).map(|(buf, idx)| buf[idx])
    }
}
///
#[derive(Clone, Copy)]
pub struct ArcBuffer;
impl BufferType for ArcBuffer {
    type Buffer<T: FixedSize> = Arc<[T]>;
}

impl<T: FixedSize> Collection for Rc<[T]> {
    type Item = T;

    type RefItem<'a>
        = &'a T
    where
        Self: 'a;

    type Iter<'a>
        = slice::Iter<'a, T>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }

    type IntoIter = Map<Zip<Repeat<Rc<[T]>>, Range<usize>>, fn((Rc<[T]>, usize)) -> T>;

    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        iter::repeat(self).zip(0..len).map(|(buf, idx)| buf[idx])
    }
}
///
#[derive(Clone, Copy)]
pub struct RcBuffer;
impl BufferType for RcBuffer {
    type Buffer<T: FixedSize> = Rc<[T]>;
}

impl<'a, T: FixedSize> Collection for &'a [T] {
    type Item = T;

    type RefItem<'b>
        = &'b T
    where
        Self: 'b;

    type Iter<'b>
        = slice::Iter<'b, T>
    where
        Self: 'b;

    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }

    type IntoIter = Copied<<&'a [T] as IntoIterator>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        <&[T] as IntoIterator>::into_iter(self).copied()
    }
}
///
#[derive(Clone, Copy)]
pub struct SliceBuffer<'a>(PhantomData<&'a ()>);
impl<'a> BufferType for SliceBuffer<'a> {
    type Buffer<T: FixedSize> = &'a [T];
}

impl<'a, T: FixedSize> Collection for &'a mut [T] {
    type Item = T;

    type RefItem<'b>
        = &'b T
    where
        Self: 'b;

    type Iter<'b>
        = slice::Iter<'b, T>
    where
        Self: 'b;

    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }

    type IntoIter = Copied<<&'a [T] as IntoIterator>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        <&[T] as IntoIterator>::into_iter(self).copied()
    }
}
///
#[derive(Clone, Copy)]
pub struct SliceMutBuffer<'a>(PhantomData<&'a ()>);
impl<'a> BufferType for SliceMutBuffer<'a> {
    type Buffer<T: FixedSize> = &'a mut [T];
}

///
pub struct Bitmap<Buffer: BufferType = VecBuffer> {
    /// The bits are stored in this buffer of bytes.
    pub(crate) buffer: Buffer::Buffer<u8>,

    /// The number of bits stored in the bitmap.
    pub(crate) _bits: usize,

    /// An offset (in number of bits) in the buffer. This enables zero-copy
    /// slicing of the bitmap on non-byte boundaries.
    pub(crate) _offset: usize,
}

impl<'a, Buffer: BufferType> IntoIterator for &'a Bitmap<Buffer> {
    type Item = &'static bool;

    type IntoIter = std::iter::Once<&'static bool>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(&true)
    }
}

impl<Buffer: BufferType> IntoIterator for Bitmap<Buffer> {
    type Item = bool;

    type IntoIter =
        Map<<<Buffer as BufferType>::Buffer<u8> as Collection>::IntoIter, fn(u8) -> bool>;

    fn into_iter(self) -> Self::IntoIter {
        Collection::into_iter(self.buffer).map(|x| x != 0)
    }
}

impl<Buffer: BufferType> Length for Bitmap<Buffer> {
    fn len(&self) -> usize {
        self._bits
    }
}

impl<Buffer: BufferType> Collection for Bitmap<Buffer> {
    type Item = bool;

    type RefItem<'a>
        = &'static bool
    where
        Self: 'a;

    type Iter<'a>
        = Map<slice::Iter<'a, u8>, fn(&'a u8) -> &'static bool>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        self.buffer
            .as_slice()
            .iter()
            .map(|x| if *x != 0 { &true } else { &false })
    }

    type IntoIter = <Self as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

///
pub struct Validity<T: Collection, Buffer: BufferType> {
    ///
    collection: T,
    ///
    bitmap: Bitmap<Buffer>,
}

impl<T: Collection, Buffer: BufferType> Length for Validity<T, Buffer> {
    fn len(&self) -> usize {
        self.bitmap.len()
    }
}

impl<T: Collection, Buffer: BufferType> Collection for Validity<T, Buffer> {
    type Item = Option<<T as Collection>::Item>;

    type RefItem<'a>
        = Option<<T as Collection>::RefItem<'a>>
    where
        Self: 'a;

    type Iter<'a>
        = Map<
        Zip<<&'a Bitmap<Buffer> as IntoIterator>::IntoIter, <T as Collection>::Iter<'a>>,
        fn(
            (&'static bool, <T as Collection>::RefItem<'a>),
        ) -> Option<<T as Collection>::RefItem<'a>>,
    >
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        self.into_iter()
    }

    type IntoIter = <Self as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<T: Collection, Buffer: BufferType> IntoIterator for Validity<T, Buffer> {
    type Item = Option<<T as Collection>::Item>;

    type IntoIter = Map<
        Zip<<Bitmap<Buffer> as IntoIterator>::IntoIter, <T as Collection>::IntoIter>,
        fn((bool, <T as Collection>::Item)) -> Option<<T as Collection>::Item>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(self.bitmap)
            .zip(Collection::into_iter(self.collection))
            .map(|(x, y)| if x { Some(y) } else { None })
    }
}

impl<'a, T: Collection, Buffer: BufferType> IntoIterator for &'a Validity<T, Buffer> {
    type Item = Option<<T as Collection>::RefItem<'a>>;

    type IntoIter = Map<
        Zip<<&'a Bitmap<Buffer> as IntoIterator>::IntoIter, <T as Collection>::Iter<'a>>,
        fn(
            (&'static bool, <T as Collection>::RefItem<'a>),
        ) -> Option<<T as Collection>::RefItem<'a>>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        (&self.bitmap)
            .into_iter()
            .zip(Collection::iter(&self.collection))
            .map(|(x, y)| if *x { Some(y) } else { None })
    }
}

///
pub trait Nullability: sealed::Sealed {
    /// `true` iff this is [`Nullable`].
    const NULLABLE: bool;

    ///
    type Item<T>;

    ///
    type Collection<T: Collection, Buffer: BufferType>: Collection<
        Item = <Self as Nullability>::Item<<T as Collection>::Item>,
    >;
}

/// Private module for [`sealed::Sealed`] trait.
mod sealed {
    /// Used to seal [`super::Nullability`].
    pub trait Sealed {}

    /// Prevent downstream implementation of [`super::Nullability`].
    impl<T> Sealed for T where T: super::Nullability {}
}

///
#[derive(Clone, Copy, Debug)]
pub struct Nullable;

impl Nullability for Nullable {
    const NULLABLE: bool = true;

    ///
    type Item<T> = Option<T>;

    ///
    type Collection<T: Collection, Buffer: BufferType> = Validity<T, Buffer>;
}

///
#[derive(Clone, Copy, Debug)]
pub struct NonNullable;

impl Nullability for NonNullable {
    const NULLABLE: bool = false;

    /// Non-nullable items are just `T`.
    type Item<T> = T;

    /// Non-nullable collections are just `T`.
    type Collection<T: Collection, Buffer: BufferType> = T;
}

///
pub struct FixedSizePrimitive<T: FixedSize, Nullable: Nullability, Buffer: BufferType>(
    Nullable::Collection<Buffer::Buffer<T>, Buffer>,
);

impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType> Length
    for FixedSizePrimitive<T, Nullable, Buffer>
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType> Collection
    for FixedSizePrimitive<T, Nullable, Buffer>
{
    type Item = Nullable::Item<T>;

    type RefItem<'a>
        = <Nullable::Collection<Buffer::Buffer<T>, Buffer> as Collection>::RefItem<'a>
    where
        Self: 'a;

    type Iter<'a>
        = <&'a Self as IntoIterator>::IntoIter
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self)
    }

    type IntoIter = <Self as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType> IntoIterator
    for FixedSizePrimitive<T, Nullable, Buffer>
{
    type Item = Nullable::Item<T>;
    type IntoIter = <Nullable::Collection<Buffer::Buffer<T>, Buffer> as Collection>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Collection::into_iter(self.0)
    }
}

impl<'a, T: FixedSize, Nullable: Nullability, Buffer: BufferType> IntoIterator
    for &'a FixedSizePrimitive<T, Nullable, Buffer>
{
    type Item = <Nullable::Collection<Buffer::Buffer<T>, Buffer> as Collection>::RefItem<'a>;
    type IntoIter = <Nullable::Collection<Buffer::Buffer<T>, Buffer> as Collection>::Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Collection::iter(&self.0)
    }
}

///
pub trait PhysicalMemoryLayout: Collection {}
impl<T: FixedSize, Nullable: Nullability, Buffer: BufferType> PhysicalMemoryLayout
    for FixedSizePrimitive<T, Nullable, Buffer>
{
}
///
pub trait Layout {
    ///
    type Layout<Buffer: BufferType, Nullable: Nullability>: PhysicalMemoryLayout; // + IntoIterator<Item = Self>;
}

impl Layout for u8 {
    type Layout<Buffer: BufferType, Nullable: Nullability> =
        FixedSizePrimitive<u8, Nullable, Buffer>;
}
// /// option maps to nullable, just maps to non-nullable
// pub struct Just<T>(PhantomData<T>);
// impl<T: Layout> Layout for Just<T> {
//     type Layout<Buffer: BufferType, Nullable: Nullability> =
//         <T as Layout>::Layout<Buffer, NonNullable>;
// }
/// a blanket impl for option here prevents downstream challenges
impl<T: Layout> Layout for Option<T> {
    type Layout<Buffer: BufferType, Nullable: Nullability> =
        <T as Layout>::Layout<Buffer, self::Nullable>;
}

///
pub struct Array<T: Layout, Nullable: Nullability<Item<T>: Layout>, Buffer: BufferType>(
    <<Nullable as Nullability>::Item<T> as Layout>::Layout<Buffer, Nullable>,
);

impl<T: Layout, Nullable: Nullability<Item<T>: Layout>, Buffer: BufferType> IntoIterator
    for Array<T, Nullable, Buffer>
{
    type Item =
        <<<Nullable as Nullability>::Item<T> as Layout>::Layout<Buffer, Nullable> as Collection>::Item;
    type IntoIter =
        <<<Nullable as Nullability>::Item<T> as Layout>::Layout<Buffer, Nullable> as Collection>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Collection::into_iter(self.0)
    }
}

impl<T: Layout, Nullable: Nullability<Item<T>: Layout>, Buffer: BufferType> Length
    for Array<T, Nullable, Buffer>
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, T: Layout, Nullable: Nullability<Item<T>: Layout>, Buffer: BufferType> IntoIterator
    for &'a Array<T, Nullable, Buffer>
{
    type Item =
        <<<Nullable as Nullability>::Item<T> as Layout>::Layout<Buffer, Nullable> as Collection>::RefItem<'a>;
    type IntoIter =
        <<<Nullable as Nullability>::Item<T> as Layout>::Layout<Buffer, Nullable> as Collection>::Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Collection::iter(&self.0)
    }
}

///
pub fn a() -> Array<u8, NonNullable, VecBuffer> {
    Array(FixedSizePrimitive(vec![1u8, 2, 3, 4]))
}

///
pub fn b() -> Array<u8, Nullable, VecBuffer> {
    Array(FixedSizePrimitive(Validity {
        collection: vec![1u8, 2, 3, 4],
        bitmap: Bitmap {
            buffer: vec![0b00001111],
            _bits: 4,
            _offset: 0,
        },
    }))
}
///
pub fn c() {
    let _: Vec<&u8> = (&a()).into_iter().collect();
    let _: Vec<u8> = a().into_iter().collect();
    let _: Vec<Option<&u8>> = (&b()).into_iter().collect();
    let _: Vec<Option<u8>> = b().into_iter().collect();
}

// ///
// pub trait Buffer<T: FixedSize>: Length {
//     /// Extracts a slice containing the entire buffer.
//     fn as_slice(&self) -> &[T];

//     /// Returns the contents of the entire buffer as a byte slice.
//     fn as_bytes(&self) -> &[u8] {
//         // Safety:
//         // - The pointer returned by slice::as_ptr (via Borrow) points to slice::len()
//         //   consecutive properly initialized values of type T, with size_of::<T> bytes
//         //   per element.
//         unsafe {
//             slice::from_raw_parts(
//                 self.as_slice().as_ptr().cast(),
//                 mem::size_of_val(self.as_slice()),
//             )
//         }
//     }

//     /// Returns the value at given index. Returns `None` if the index is out of range.
//     fn index(&self, index: usize) -> Option<&T> {
//         self.as_slice().get(index)
//     }

//     /// Returns the value at given index. Skips bound checking.
//     ///
//     /// # Safety
//     ///
//     /// Caller must ensure index is within bounds.
//     unsafe fn index_unchecked(&self, index: usize) -> &T {
//         self.as_slice().get_unchecked(index)
//     }

//     /// Returns an iterator that borrows the items in the buffer.
//     fn iter(&self) -> slice::Iter<'_, T> {
//         self.as_slice().iter()
//     }

//     /// The owned iterator for this buffer.
//     type IntoIter: Iterator<Item = T>;

//     /// Move the buffer into an iterator.
//     fn into_iter(self) -> Self::IntoIter;
// }

// ///
// pub trait BufferMut<T: FixedSize>: Buffer<T> {}
// ///
// pub trait BufferAlloc<T: FixedSize>: BufferMut<T> {}
