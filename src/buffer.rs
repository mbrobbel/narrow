//! Traits for memory buffers.

use crate::{nullability::Collection, FixedSize, Length};
use std::{
    iter::{self, Copied, Map, Repeat, Zip},
    marker::PhantomData,
    mem,
    ops::Range,
    rc::Rc,
    slice,
    sync::Arc,
};

/// A memory buffer type constructor for Arrow data.
///
/// The generic associated type constructor [`Self::Buffer`] defines the
/// [`Buffer`] type that stores [`FixedSize`] items.
///
// note
// Arrow buffers are like Rust slices with "primitive" item types.
// Another way to implement the buffer trait: a subtrait of Borrow<[T]> and then
// implement Buffer<T> for all U: Borrow<[T] where T: FixedSize, however,the approach here is a little
// bit more elaborate to also support buffer types that don't implement Borrow<[T]>.
pub trait BufferType {
    /// A [`Buffer`] type for [`FixedSize`] items of type `T`.
    type Buffer<T: FixedSize>: Buffer<T>;
}

/// A contiguous immutable memory buffer for Arrow data.
pub trait Buffer<T: FixedSize>: Collection<Item = T> {
    /// Extracts a slice containing the entire buffer.
    fn as_slice(&self) -> &[T];

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
}

/// An immutable reference to a buffer.
///
/// This can be used to provide immutable access to an internal buffer.
pub trait BufferRef<T: FixedSize> {
    /// The [Buffer] type.
    type Buffer: Buffer<T>;

    /// Returns an immutable reference to a buffer.
    fn buffer_ref(&self) -> &Self::Buffer;
}

/// A mutable reference to a buffer.
///
/// This can be used to provide mutable access to an internal buffer.
pub trait BufferRefMut<T: FixedSize> {
    /// The [`BufferMut`] type.
    type BufferMut: BufferMut<T>;

    /// Returns a mutable reference to a buffer.
    fn buffer_ref_mut(&mut self) -> &mut Self::BufferMut;
}

/// A contiguous mutable memory buffer for Arrow data.
pub trait BufferMut<T: FixedSize>: Buffer<T> {
    /// Extracts a mutable slice containing the entire buffer.
    fn as_mut_slice(&mut self) -> &mut [T];

    /// Returns the contents of the entire buffer as a mutable byte slice.
    fn as_mut_bytes(&mut self) -> &mut [u8] {
        // Safety:
        // - The pointer returned by slice::as_mut_ptr (via Borrow) points to slice::len()
        //   consecutive properly initialized values of type T, with size_of::<T> bytes
        //   per element.
        unsafe {
            slice::from_raw_parts_mut(
                self.as_mut_slice().as_mut_ptr().cast(),
                mem::size_of_val(self.as_slice()),
            )
        }
    }

    /// Iterate over the items in this buffer using a mutable slice iterator.
    fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }
}

/// A [`BufferType`] for a single item.
#[derive(Clone, Copy, Debug)]
pub struct SingleBuffer;

impl BufferType for SingleBuffer {
    type Buffer<T: FixedSize> = <ArrayBuffer<1> as BufferType>::Buffer<T>;
}

impl<T: FixedSize, const N: usize> Collection for [T; N] {
    type Item = T;
    type RefItem<'a>
        = &'a T
    where
        Self: 'a;

    type Iter<'a>
        = <&'a [T; N] as IntoIterator>::IntoIter
    where
        Self: 'a;
    type IntoIter = <[T; N] as IntoIterator>::IntoIter;

    fn iter(&self) -> Self::Iter<'_> {
        <&[T; N] as IntoIterator>::into_iter(self)
    }

    fn into_iter(self) -> Self::IntoIter {
        <[T; N] as IntoIterator>::into_iter(self)
    }
}

impl<'a, T: FixedSize> Collection for &'a [T] {
    type Item = T;
    type RefItem<'b>
        = &'b T
    where
        Self: 'b;

    type Iter<'b>
        = <&'b [T] as IntoIterator>::IntoIter
    where
        Self: 'b;
    type IntoIter = Copied<<&'a [T] as IntoIterator>::IntoIter>;

    fn iter(&self) -> Self::Iter<'_> {
        <&[T] as IntoIterator>::into_iter(self)
    }

    fn into_iter(self) -> Self::IntoIter {
        <&[T] as IntoIterator>::into_iter(self).copied()
    }
}

impl<'a, T: FixedSize> Collection for &'a mut [T] {
    type Item = T;
    type RefItem<'b>
        = &'b T
    where
        Self: 'b;

    type Iter<'b>
        = <&'b [T] as IntoIterator>::IntoIter
    where
        Self: 'b;
    type IntoIter = Copied<<&'a [T] as IntoIterator>::IntoIter>;

    fn iter(&self) -> Self::Iter<'_> {
        <&[T] as IntoIterator>::into_iter(self)
    }

    fn into_iter(self) -> Self::IntoIter {
        <&[T] as IntoIterator>::into_iter(self).copied()
    }
}

/// A [`BufferType`] implementation for array.
///
/// Stores items `T` in `[T; N]`.
#[derive(Clone, Copy, Debug)]
pub struct ArrayBuffer<const N: usize>;

impl<const N: usize> BufferType for ArrayBuffer<N> {
    type Buffer<T: FixedSize> = [T; N];
}

impl<T: FixedSize, const N: usize> Buffer<T> for [T; N] {
    fn as_slice(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: FixedSize, const N: usize> BufferMut<T> for [T; N] {
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

// /// A [`BufferType`] implementation for an array of arrays.
// ///
// /// Stores items `T` in `[[T; N]; M]`.
// #[cfg(not(feature = "arrow-rs"))]
// #[derive(Clone, Copy, Debug)]
// pub struct ArrayArrayBuffer<const N: usize, const M: usize>;

// #[cfg(not(feature = "arrow-rs"))]
// impl<const N: usize, const M: usize> BufferType for ArrayArrayBuffer<N, M> {
//     type Buffer<T: FixedSize> = [[T; N]; M];
// }

// #[cfg(not(feature = "arrow-rs"))]
// impl<T: FixedSize, const N: usize, const M: usize> Buffer<T> for [[T; N]; M] {
//     fn as_slice(&self) -> &[T] {
//         // self.flatten() is nightly
//         // SAFETY: `[T]` is layout-identical to `[[T; M]; N]`
//         unsafe { std::slice::from_raw_parts(self.as_ptr().cast(), M * N) }
//     }
// }

// #[cfg(not(feature = "arrow-rs"))]
// impl<T: FixedSize, const N: usize, const M: usize> BufferMut<T> for [[T; N]; M] {
//     fn as_mut_slice(&mut self) -> &mut [T] {
//         // self.flatten() is nightly
//         // SAFETY: `[T]` is layout-identical to `[[T; M]; N]`
//         unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr().cast(), M * N) }
//     }
// }

/// A [`BufferType`] implementation for slice.
///
/// Stores items `T` in `&[T]`.
#[derive(Clone, Copy, Debug)]
pub struct SliceBuffer<'a>(PhantomData<&'a ()>);

impl<'a> BufferType for SliceBuffer<'a> {
    type Buffer<T: FixedSize> = &'a [T];
}

impl<'a, T: FixedSize> Buffer<T> for &'a [T] {
    fn as_slice(&self) -> &[T] {
        self
    }
}

/// A [`BufferType`] implementation for mutable slice.
///
/// Stores items `T` in `&mut [T]`.
#[derive(Clone, Copy, Debug)]
pub struct SliceMutBuffer<'a>(PhantomData<&'a ()>);

impl<'a> BufferType for SliceMutBuffer<'a> {
    type Buffer<T: FixedSize> = &'a mut [T];
}

impl<'a, T: FixedSize> Buffer<T> for &'a mut [T] {
    fn as_slice(&self) -> &[T] {
        self
    }
}

impl<T: FixedSize> BufferMut<T> for &mut [T] {
    fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }
}

// impl<'a, T: FixedSize, const N: usize> Collection for &'a [[T; N]] {
//     type Item = T;
//     type RefItem<'b>
//         = &'b T
//     where
//         Self: 'b;

//     type Iter<'b>
//         = Flatten<<&'b [[T; N]] as IntoIterator>::IntoIter>
//     where
//         Self: 'b;
//     type IntoIter = Copied<Flatten<<&'a [[T; N]] as IntoIterator>::IntoIter>>;

//     fn iter(&self) -> Self::Iter<'_> {
//         <&[[T; N]] as IntoIterator>::into_iter(self).flatten()
//     }

//     fn into_iter(self) -> Self::IntoIter {
//         <&[[T; N]] as IntoIterator>::into_iter(self)
//             .flatten()
//             .copied()
//     }
// }

// impl<'a, T: FixedSize, const N: usize> Collection for &'a mut [[T; N]] {
//     type Item = T;
//     type RefItem<'b>
//         = &'b T
//     where
//         Self: 'b;

//     type Iter<'b>
//         = Flatten<<&'b [[T; N]] as IntoIterator>::IntoIter>
//     where
//         Self: 'b;
//     type IntoIter = Copied<Flatten<<&'a [[T; N]] as IntoIterator>::IntoIter>>;

//     fn iter(&self) -> Self::Iter<'_> {
//         <&[[T; N]] as IntoIterator>::into_iter(self).flatten()
//     }

//     fn into_iter(self) -> Self::IntoIter {
//         <&[[T; N]] as IntoIterator>::into_iter(self)
//             .flatten()
//             .copied()
//     }
// }

// /// A [`BufferType`] implementation for slice with array items.
// ///
// /// Stores items `T` in `&[[T; N]]`.
// #[derive(Clone, Copy, Debug)]
// pub struct SliceArrayBuffer<'a, const N: usize>(PhantomData<&'a ()>);

// impl<'a, const N: usize> BufferType for SliceArrayBuffer<'a, N> {
//     type Buffer<T: FixedSize> = &'a [[T; N]];
// }

// impl<'a, T: FixedSize, const N: usize> Buffer<T> for &'a [[T; N]] {
//     fn as_slice(&self) -> &[T] {
//         // self.flatten() is nightly
//         // SAFETY: `[T]` is layout-identical to `[T; N]`
//         unsafe { std::slice::from_raw_parts(self.as_ptr().cast(), <[[T; N]]>::len(self) * N) }
//     }
// }

// /// A [`BufferType`] implementation for mutable slice with array items.
// ///
// /// Stores items `T` in `&mut [[T; N]]`.
// #[derive(Clone, Copy, Debug)]
// pub struct SliceArrayMutBuffer<'a, const N: usize>(PhantomData<&'a ()>);

// impl<'a, const N: usize> BufferType for SliceArrayMutBuffer<'a, N> {
//     type Buffer<T: FixedSize> = &'a mut [[T; N]];
// }

// impl<'a, T: FixedSize, const N: usize> Buffer<T> for &'a mut [[T; N]] {
//     fn as_slice(&self) -> &[T] {
//         // self.flatten() is nightly
//         // SAFETY: `[T]` is layout-identical to `[T; N]`
//         unsafe { std::slice::from_raw_parts(self.as_ptr().cast(), <[[T; N]]>::len(self) * N) }
//     }
// }

// impl<T: FixedSize, const N: usize> BufferMut<T> for &mut [[T; N]] {
//     fn as_mut_slice(&mut self) -> &mut [T] {
//         // self.flatten() is nightly
//         // SAFETY: `[T]` is layout-identical to `[T; N]`
//         unsafe {
//             std::slice::from_raw_parts_mut(self.as_mut_ptr().cast(), <[[T; N]]>::len(self) * N)
//         }
//     }
// }

/// A [`BufferType`] implementation for [`Vec`].
///
/// Stores items `T` in `Vec<T>`.
#[derive(Clone, Copy, Debug)]
pub struct VecBuffer;

impl BufferType for VecBuffer {
    type Buffer<T: FixedSize> = Vec<T>;
}

impl<T: FixedSize> Collection for Vec<T> {
    type Item = T;
    type RefItem<'a>
        = &'a T
    where
        Self: 'a;

    type Iter<'a>
        = <&'a Vec<T> as IntoIterator>::IntoIter
    where
        Self: 'a;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn iter(&self) -> Self::Iter<'_> {
        <&Vec<T> as IntoIterator>::into_iter(self)
    }

    fn into_iter(self) -> Self::IntoIter {
        <Vec<T> as IntoIterator>::into_iter(self)
    }
}

impl<T: FixedSize> Buffer<T> for Vec<T> {
    fn as_slice(&self) -> &[T] {
        self.as_slice()
    }
}

// impl<T: FixedSize, const N: usize> Buffer<T> for Vec<[T; N]> {
//     fn as_slice(&self) -> &[T] {
//         // self.flatten() is nightly
//         // SAFETY: `[T]` is layout-identical to `[T; N]`
//         unsafe { std::slice::from_raw_parts(self.as_ptr().cast(), Vec::<[T; N]>::len(self) * N) }
//     }
// }

impl<T: FixedSize> BufferMut<T> for Vec<T> {
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

// impl<T: FixedSize, const N: usize> BufferMut<T> for Vec<[T; N]> {
//     fn as_mut_slice(&mut self) -> &mut [T] {
//         // self.flatten() is nightly
//         // SAFETY: `[T]` is layout-identical to `[T; N]`
//         unsafe {
//             std::slice::from_raw_parts_mut(self.as_mut_ptr().cast(), Vec::<[T; N]>::len(self) * N)
//         }
//     }
// }

// /// A [`BufferType`] implementation for [`Vec`] with array items.
// ///
// /// Stores items `T` in `Vec<[T;N]>`.
// #[derive(Clone, Copy, Debug)]
// pub struct VecArrayBuffer<const N: usize>;

// impl<const N: usize> BufferType for VecArrayBuffer<N> {
//     type Buffer<T: FixedSize> = Vec<[T; N]>;
// }

// impl<T: FixedSize, const N: usize> Buffer<T> for Vec<[T; N]> {
//     fn as_slice(&self) -> &[T] {
//         // self.flatten() is nightly
//         // SAFETY: `[T]` is layout-identical to `[T; N]`
//         unsafe { std::slice::from_raw_parts(self.as_ptr().cast(), Vec::<[T; N]>::len(self) * N) }
//     }
// }

// impl<T: FixedSize, const N: usize> Collection for Vec<[T; N]> {
//     type Item = T;
//     type RefItem<'a>
//         = &'a T
//     where
//         Self: 'a;

//     type Iter<'a>
//         = Flatten<<&'a Vec<[T; N]> as IntoIterator>::IntoIter>
//     where
//         Self: 'a;
//     type IntoIter = Flatten<<Vec<[T; N]> as IntoIterator>::IntoIter>;

//     fn iter(&self) -> Self::Iter<'_> {
//         <&Vec<[T; N]> as IntoIterator>::into_iter(self).flatten()
//     }

//     fn into_iter(self) -> Self::IntoIter {
//         <Vec<[T; N]> as IntoIterator>::into_iter(self).flatten()
//     }
// }

// impl<T: FixedSize, const N: usize> BufferMut<T> for Vec<[T; N]> {
//     fn as_mut_slice(&mut self) -> &mut [T] {
//         // self.flatten() is nightly
//         // SAFETY: `[T]` is layout-identical to `[T; N]`
//         unsafe {
//             std::slice::from_raw_parts_mut(self.as_mut_ptr().cast(), Vec::<[T; N]>::len(self) * N)
//         }
//     }
// }

/// A [`BufferType`] implementation for [`Box`].
///
/// Stores items `T` in `Box<[T]>`.
#[derive(Clone, Copy, Debug)]
pub struct BoxBuffer;

impl BufferType for BoxBuffer {
    type Buffer<T: FixedSize> = Box<[T]>;
}

impl<T: FixedSize> Collection for Box<[T]> {
    type Item = T;
    type RefItem<'a>
        = &'a T
    where
        Self: 'a;

    type Iter<'a>
        = <&'a Box<[T]> as IntoIterator>::IntoIter
    where
        Self: 'a;
    type IntoIter = <Box<[T]> as IntoIterator>::IntoIter;

    fn iter(&self) -> Self::Iter<'_> {
        <&Box<[T]> as IntoIterator>::into_iter(self)
    }

    fn into_iter(self) -> Self::IntoIter {
        <Box<[T]> as IntoIterator>::into_iter(self)
    }
}

impl<T: FixedSize> Buffer<T> for Box<[T]> {
    fn as_slice(&self) -> &[T] {
        <&[T]>::from(self)
    }
}

impl<T: FixedSize> BufferMut<T> for Box<[T]> {
    fn as_mut_slice(&mut self) -> &mut [T] {
        <&mut [T]>::from(self)
    }
}

/// A [`BufferType`] implementation for [`Arc`].
///
/// Stores items `T` in `Arc<[T]>`.
#[derive(Clone, Copy, Debug)]
pub struct ArcBuffer;

impl BufferType for ArcBuffer {
    type Buffer<T: FixedSize> = Arc<[T]>;
}

impl<T: FixedSize> Collection for Arc<[T]> {
    type Item = T;
    type RefItem<'a>
        = &'a T
    where
        Self: 'a;

    type Iter<'a>
        = <&'a [T] as IntoIterator>::IntoIter
    where
        Self: 'a;
    type IntoIter = Map<Zip<Repeat<Arc<[T]>>, Range<usize>>, fn((Arc<[T]>, usize)) -> T>;

    fn iter(&self) -> Self::Iter<'_> {
        <&[T] as IntoIterator>::into_iter(self)
    }

    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        iter::repeat(self).zip(0..len).map(|(buf, idx)| buf[idx])
    }
}

impl<T: FixedSize> Buffer<T> for Arc<[T]> {
    fn as_slice(&self) -> &[T] {
        <&[T]>::from(self)
    }
}

impl<T: FixedSize> BufferMut<T> for Arc<[T]> {
    fn as_mut_slice(&mut self) -> &mut [T] {
        match Arc::get_mut(self) {
            Some(slice) => slice,
            None => panic!("not safe to mutate shared value"),
        }
    }
}

/// A [`BufferType`] implementation for [`Rc`].
///
/// Stores items `T` in `Rc<[T]>`.
#[derive(Clone, Copy, Debug)]
pub struct RcBuffer;

impl BufferType for RcBuffer {
    type Buffer<T: FixedSize> = Rc<[T]>;
}

impl<T: FixedSize> Collection for Rc<[T]> {
    type Item = T;
    type RefItem<'a>
        = &'a T
    where
        Self: 'a;

    type Iter<'a>
        = <&'a [T] as IntoIterator>::IntoIter
    where
        Self: 'a;
    type IntoIter = Map<Zip<Repeat<Rc<[T]>>, Range<usize>>, fn((Rc<[T]>, usize)) -> T>;

    fn iter(&self) -> Self::Iter<'_> {
        <&[T] as IntoIterator>::into_iter(self)
    }

    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        iter::repeat(self).zip(0..len).map(|(buf, idx)| buf[idx])
    }
}

impl<T: FixedSize> Buffer<T> for Rc<[T]> {
    fn as_slice(&self) -> &[T] {
        <&[T]>::from(self)
    }
}

impl<T: FixedSize> BufferMut<T> for Rc<[T]> {
    fn as_mut_slice(&mut self) -> &mut [T] {
        match Rc::get_mut(self) {
            Some(slice) => slice,
            None => panic!("not safe to mutate shared value"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single() {
        let mut single: <SingleBuffer as BufferType>::Buffer<u16> = [1234];
        assert_eq!(single.as_bytes(), [210, 4]);
        single.as_mut_bytes()[1] = 0;
        assert_eq!(single.as_bytes(), [210, 0]);
        single.as_mut_slice()[0] = 1234;
        assert_eq!(single, [1234]);
    }

    #[test]
    fn array() {
        let mut array: <ArrayBuffer<4> as BufferType>::Buffer<u16> = [1, 2, 3, 4];
        assert_eq!(
            <_ as Buffer<u16>>::as_bytes(&array),
            &[1, 0, 2, 0, 3, 0, 4, 0]
        );
        <_ as BufferMut<u16>>::as_mut_bytes(&mut array)[1] = 1;
        assert_eq!(<_ as Buffer<u16>>::as_bytes(&array)[..2], [1, 1]);
        array.as_mut_slice()[0] = 1;
        assert_eq!(array, [1, 2, 3, 4]);
    }

    // #[test]
    // #[cfg(not(feature = "arrow-rs"))]
    // fn array_array() {
    //     let mut array_array: <ArrayArrayBuffer<2, 4> as BufferType>::Buffer<u8> =
    //         [[1, 2], [3, 4], [1, 2], [3, 4]];
    //     assert_eq!(
    //         <_ as Buffer<u8>>::as_bytes(&array_array),
    //         &[1, 2, 3, 4, 1, 2, 3, 4]
    //     );
    //     <_ as BufferMut<u8>>::as_mut_bytes(&mut array_array)[1] = 1;
    //     assert_eq!(
    //         <_ as Buffer<u8>>::as_slice(&array_array),
    //         [1, 1, 3, 4, 1, 2, 3, 4]
    //     );
    // }

    #[test]
    fn slice() {
        let slice: <SliceBuffer as BufferType>::Buffer<u16> = &[1234, 4321];
        assert_eq!(slice.as_bytes(), &[210, 4, 225, 16]);
        let mut slice_mut: <SliceMutBuffer as BufferType>::Buffer<u16> = &mut [4321, 1234];
        slice_mut.as_mut_slice()[0] = 1234;
        slice_mut.as_mut_slice()[1] = 4321;
        assert_eq!(slice, slice_mut);
    }

    // #[test]
    // fn slice_array() {
    //     let slice_array: <SliceArrayBuffer<2> as BufferType>::Buffer<u32> = &[[1, 2], [3, 4]];
    //     assert_eq!(<_ as Buffer<u32>>::as_slice(&slice_array), [1, 2, 3, 4]);
    //     let mut slice_array_mut: <SliceArrayMutBuffer<3> as BufferType>::Buffer<u8> =
    //         &mut [[1, 2, 3], [4, 5, 6]];
    //     slice_array_mut.as_mut_slice()[0] = 0;
    //     assert_eq!(
    //         <_ as Buffer<u8>>::as_bytes(&slice_array_mut),
    //         &[0, 2, 3, 4, 5, 6]
    //     );
    // }
}
