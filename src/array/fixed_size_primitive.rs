use crate::{Array, ArrayData, ArrayIndex, Buffer, Nullable, Primitive, Validity, ALIGNMENT};
use paste::paste;
use std::{iter::FromIterator, ops::Deref};

/// Array with primitive values.
#[derive(Debug)]
pub struct FixedSizePrimitiveArray<T, const N: bool>(Validity<Buffer<T, ALIGNMENT>, N>)
where
    T: Primitive;

impl<T, const N: bool> Array for FixedSizePrimitiveArray<T, N>
where
    T: Primitive,
{
    type Validity = Validity<Buffer<T, ALIGNMENT>, N>;

    fn validity(&self) -> &Self::Validity {
        &self.0
    }
}

impl<T> ArrayIndex<usize> for FixedSizePrimitiveArray<T, false>
where
    T: Primitive,
{
    type Output = T;

    fn index(&self, index: usize) -> Self::Output {
        self.0[index]
    }
}

impl<T> ArrayIndex<usize> for FixedSizePrimitiveArray<T, true>
where
    T: Primitive,
{
    type Output = Option<T>;

    fn index(&self, index: usize) -> Self::Output {
        if self.0.is_valid(index) {
            Some(self.0.data()[index])
        } else {
            None
        }
    }
}

impl<T> Deref for FixedSizePrimitiveArray<T, false>
where
    T: Primitive,
{
    type Target = Buffer<T, ALIGNMENT>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Deref for FixedSizePrimitiveArray<T, true>
where
    T: Primitive,
{
    type Target = Nullable<Buffer<T, ALIGNMENT>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! impl_primitive {
    ($ident:ident, $ty:ty) => {
        paste! {
                #[doc = "Array with [" $ty "] values."]
                pub type $ident<const N: bool> = FixedSizePrimitiveArray<$ty, N>;

                impl crate::ArrayType for $ty {
                    type Array = FixedSizePrimitiveArray<$ty, false>;
                }

                impl crate::ArrayType for Option<$ty> {
                    type Array = FixedSizePrimitiveArray<$ty, true>;
                }
        }
    };
}

macro_rules! impl_ptr_width {
    ($ty:ty, $char:expr, $value:expr) => {
        paste! {
            impl crate::ArrayType for $ty {
                #[cfg(any(doc, target_pointer_width = "16"))]
                /// When `target_pointer_width` is 16.
                type Array = FixedSizePrimitiveArray<[<$char "16">], $value>;
                #[cfg(any(doc, target_pointer_width = "32"))]
                /// When `target_pointer_width` is 32.
                type Array = FixedSizePrimitiveArray<[<$char "32">], $value>;
                #[cfg(any(doc, target_pointer_width = "64"))]
                /// When `target_pointer_width` is 64.
                type Array = FixedSizePrimitiveArray<[<$char "64">], $value>;
            }
        }
    };
}

impl_primitive!(Int8Array, i8);
impl_primitive!(Int16Array, i16);
impl_primitive!(Int32Array, i32);
impl_primitive!(Int64Array, i64);
impl_primitive!(Uint8Array, u8);
impl_primitive!(Uint16Array, u16);
impl_primitive!(Uint32Array, u32);
impl_primitive!(Uint64Array, u64);
impl_primitive!(Float32Array, f32);
impl_primitive!(Float64Array, f64);

impl_ptr_width!(isize, 'i', false);
impl_ptr_width!(usize, 'u', false);
impl_ptr_width!(Option<isize>, 'i', true);
impl_ptr_width!(Option<usize>, 'u', true);

impl<T> FromIterator<T> for FixedSizePrimitiveArray<T, false>
where
    T: Primitive,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<'a, T> FromIterator<&'a T> for FixedSizePrimitiveArray<T, false>
where
    T: Primitive + 'a,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a T>,
    {
        Self(iter.into_iter().copied().collect())
    }
}

impl<'a, T> FromIterator<Option<&'a T>> for FixedSizePrimitiveArray<T, true>
where
    T: Primitive,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<&'a T>>,
    {
        Self(iter.into_iter().map(|opt| opt.copied()).collect())
    }
}

impl<'a, T> FromIterator<&'a Option<T>> for FixedSizePrimitiveArray<T, true>
where
    T: Primitive + 'a,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a Option<T>>,
    {
        Self(iter.into_iter().copied().collect())
    }
}

impl<T> FromIterator<Option<T>> for FixedSizePrimitiveArray<T, true>
where
    T: Primitive,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<T>>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<'a, T> IntoIterator for &'a FixedSizePrimitiveArray<T, false>
where
    T: Primitive,
{
    type Item = T;
    type IntoIter = <&'a Buffer<T, ALIGNMENT> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a FixedSizePrimitiveArray<T, true>
where
    T: Primitive,
{
    type Item = Option<T>;
    type IntoIter = <&'a Nullable<Buffer<T, ALIGNMENT>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter() {
        let array = [1u8, 2, 3, 4].iter().collect::<Uint8Array<false>>();
        assert_eq!(&array[..], &[1, 2, 3, 4]);
        assert_eq!(Array::len(&array), 4);
        assert_eq!(Array::valid_count(&array), 4);
        assert_eq!(Array::null_count(&array), 0);

        let array = [Some(1u8), None, Some(3), Some(4)]
            .iter()
            .collect::<Uint8Array<true>>();
        assert_eq!(
            &array.into_iter().collect::<Vec<_>>()[..],
            &[Some(1), None, Some(3), Some(4)]
        );
        assert_eq!(Array::len(&array), 4);
        assert_eq!(Array::valid_count(&array), 3);
        assert_eq!(Array::null_count(&array), 1);
    }
}
