/// Array with primitive values.
pub struct FixedSizePrimitiveArray<T, U = Box<[T]>, const N: bool = true>(Validity<U, N>)
where
    U: Buffer<T>,
    T: Primitive;

// /// Array with primitive values.
// #[derive(Debug, PartialEq, Eq, Hash)]
// pub struct FixedSizePrimitiveArray<T, U, const N: bool = true>(Validity<U, N>)
// where
//     U: Buffer<T>;

// impl<T, const N: bool, const A: usize> Array for FixedSizePrimitiveArray<T, N, A> {
//     type Item<'a> = T;
// }

// impl<T, const N: bool, const A: usize> Clone for FixedSizePrimitiveArray<T, N, A>
// where
//     T: Primitive,
// {
//     fn clone(&self) -> Self {
//         Self(self.0.clone())
//     }
// }

// impl<T, const N: bool, const A: usize> DataBuffer<T, A> for FixedSizePrimitiveArray<T, N, A> {
//     fn data_buffer(&self) -> &Buffer<T, A> {
//         self.0.data_buffer()
//     }
// }

// // todo(mb): remove
// impl<T, const A: usize> Deref for FixedSizePrimitiveArray<T, false, A> {
//     type Target = Validity<Buffer<T, A>, false>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl<T, const A: usize> Deref for FixedSizePrimitiveArray<T, true, A> {
//     type Target = Nullable<Buffer<T, A>>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// macro_rules! impl_primitive {
//     ($ident:ident, $ty:ty) => {
//         #[doc = "Array with ["]
//         #[doc = stringify!($ty)]
//         #[doc = "] values."]
//         pub type $ident<const N: bool = true, const A: usize = DEFAULT_ALIGNMENT> =
//             FixedSizePrimitiveArray<$ty, N, A>;

//         impl ArrayType for $ty {
//             type Item<'a> = $ty;
//             type Array<T, const N: bool, const A: usize> = FixedSizePrimitiveArray<$ty, false, A>;
//         }
//     };
// }

// impl_primitive!(Int8Array, i8);
// impl_primitive!(Int16Array, i16);
// impl_primitive!(Int32Array, i32);
// impl_primitive!(Int64Array, i64);
// impl_primitive!(Uint8Array, u8);
// impl_primitive!(Uint16Array, u16);
// impl_primitive!(Uint32Array, u32);
// impl_primitive!(Uint64Array, u64);
// impl_primitive!(Float32Array, f32);
// impl_primitive!(Float64Array, f64);

// macro_rules! impl_ptr_width {
//     ($ty:ty, $a:ty, $b:ty, $c:ty) => {
//         impl ArrayType for $ty {
//             type Item<'a> = $ty;
//             #[cfg(any(doc, target_pointer_width = "16"))]
//             /// When `target_pointer_width` is 16.
//             type Array<T, const N: bool, const A: usize> = FixedSizePrimitiveArray<$a, false, A>;
//             #[cfg(any(doc, target_pointer_width = "32"))]
//             /// When `target_pointer_width` is 32.
//             type Array<T, const N: bool, const A: usize> = FixedSizePrimitiveArray<$b, false, A>;
//             #[cfg(any(doc, target_pointer_width = "64"))]
//             /// When `target_pointer_width` is 64.
//             type Array<T, const N: bool, const A: usize> = FixedSizePrimitiveArray<$c, false, A>;
//         }
//     };
// }

// impl_ptr_width!(isize, i16, i32, i64);
// impl_ptr_width!(usize, u16, u32, u64);

// impl<T, U, const N: bool, const A: usize> FromIterator<U> for FixedSizePrimitiveArray<T, N, A>
// where
//     T: Primitive,
//     Validity<Buffer<T, A>, N>: FromIterator<U>,
// {
//     fn from_iter<I>(iter: I) -> Self
//     where
//         I: IntoIterator<Item = U>,
//     {
//         Self(iter.into_iter().collect())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::Length;
//     use crate::Null;

//     #[test]
//     fn from_iter() {
//         let array = [1u8, 2, 3, 4].into_iter().collect::<Uint8Array<false>>();
//         assert_eq!(&array[..], &[1, 2, 3, 4]);
//         assert_eq!(array.len(), 4);
//         assert_eq!(array.valid_count(), 4);
//         assert_eq!(array.null_count(), 0);

//         let array = [Some(1u8), None, Some(3), Some(4)]
//             .into_iter()
//             .collect::<Uint8Array>();
//         assert_eq!(
//             array.into_iter().collect::<Vec<_>>(),
//             &[Some(&1), None, Some(&3), Some(&4)]
//         );
//         assert_eq!(array.len(), 4);
//         assert_eq!(array.valid_count(), 3);
//         assert_eq!(array.null_count(), 1);
//     }
// }

use crate::{Buffer, Primitive};
