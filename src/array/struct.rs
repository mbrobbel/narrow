use crate::{Array, ArrayData, ArrayType, Nullable, Validity};
use std::{fmt::Debug, ops::Deref};

// todo(mb): add partial eq with array of struct

/// Struct types that can be stored in arrays.
///
/// Enables converting arrays of structs into structs of arrays.
pub trait StructArrayType: Sized {
    /// The type storing the struct of arrays.
    // ArrayData because the struct array does not contain the validity of the
    // StructArray.
    type Array: FromIterator<Self> + ArrayData;
}

// todo(mb): when GATs: type Array<const N: bool> = <T as ArrayType>::Array<true>;
// todo(mb): this conflicts with the idea to support options as unions
//           also, what about non-struct types in options like
//           Option<Vec<T>> or Option<Result<_, _>>?
// or(add generic T to ArrayType to allow impl ArrayType<T> for Option<T> in derive macro)
impl<T> ArrayType for Option<T>
where
    T: StructArrayType,
{
    type Array<const N: bool> = StructArray<T, true>;
}

/// Array with structs that have fields of other array types.
pub struct StructArray<T, const N: bool>(Validity<<T as StructArrayType>::Array, N>)
where
    T: StructArrayType;

impl<T, const N: bool> Debug for StructArray<T, N>
where
    T: StructArrayType,
    Validity<<T as StructArrayType>::Array, N>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StructArray").field(&self.0).finish()
    }
}

impl<T, const N: bool> Array for StructArray<T, N>
where
    T: StructArrayType,
{
    type Validity = Validity<<T as StructArrayType>::Array, N>;

    fn validity(&self) -> &Self::Validity {
        &self.0
    }
}

impl<T, const N: bool> Deref for StructArray<T, N>
where
    T: StructArrayType,
{
    type Target = Validity<<T as StructArrayType>::Array, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromIterator<T> for StructArray<T, false>
where
    T: StructArrayType,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<T> FromIterator<Option<T>> for StructArray<T, true>
where
    T: StructArrayType + Default,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<T>>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<'a, T> IntoIterator for &'a StructArray<T, false>
where
    T: StructArrayType,
    // need GATs to add this to StructArrayType
    &'a T::Array: IntoIterator<Item = T>,
{
    type Item = T;
    type IntoIter = <&'a T::Array as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a StructArray<T, true>
where
    T: StructArrayType,
    &'a Nullable<T::Array>: IntoIterator, //<Item = Option<T>>,
{
    type Item = <Self::IntoIter as Iterator>::Item;
    // <<&'a Nullable<<T as StructArrayType>::Array> as IntoIterator>::IntoIter as Iterator>::Item; //Option<T>;
    type IntoIter = <&'a Nullable<T::Array> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Array, Copy, Clone, Debug, Default, PartialEq)]
pub struct Unit;

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::BooleanArray;

    #[test]
    fn unit_struct() {
        let vec = vec![Unit; 123];
        let array = vec.iter().copied().collect::<UnitArray<false>>();
        assert_eq!(Array::len(&array), 123);
        assert!(Array::is_valid(&array, 0));
        assert!(!Array::is_null(&array, 122));
        assert_eq!(vec, array.into_iter().collect::<Vec<_>>());

        let vec = vec![Some(Unit); 123];
        let array = vec.iter().copied().collect::<UnitArray<true>>();
        assert_eq!(Array::len(&array), 123);
        assert!(Array::is_valid(&array, 0));
        assert!(!Array::is_null(&array, 122));
        assert_eq!(vec, array.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn unnamed_fields_struct() {
        // pub trait Bar {
        //     type X;
        // }

        // impl Bar for bool {
        //     type X = Self;
        // }

        #[derive(Array, Clone, Debug, PartialEq, Eq)]
        pub struct Unnamed<'a, T: Bar, const N: usize>(pub T, [u8; N], &'a str, Vec<T>)
        where
            T: Copy,
            <T as Bar>::X: Copy;
        // let vec = vec![Unnamed(false, [1u8], "asdf", vec![false]); 123];
        // let array = vec
        //     .iter()
        //     .cloned()
        //     .collect::<UnnamedArray<bool, 1, false>>();
        // let mut iter = array.into_iter();
        // assert!(iter.next().is_some());
        // assert_eq!(array.into_iter().collect::<Vec<_>>(), vec);

        #[derive(Array, Clone, Debug, PartialEq, Eq)]
        pub struct Unnamed<T>(T, u16, u32, u64);
        let vec = vec![Unnamed(Unit, 2, 3, 4); 4];
        let array = vec.iter().cloned().collect::<UnnamedArray<Unit, false>>();

        pub struct Foo<'a, T: ArrayType>(<&'a T::Array as IntoIterator>::IntoIter)
        where
            &'a T::Array: IntoIterator;
        // let foo = Foo(1u8);
        // pub struct UnnamedIterArray<'array, T: ArrayType>(
        //     <&'array T::Array as IntoIterator>::IntoIter,
        // )
        // where
        //     &'array T::Array: IntoIterator;

        // let iter = UnnamedIterArray(
        //     &array.0.into_iter(),
        //     //     &array.1.into_iter(),
        //     //     &array.2.into_iter(),
        //     //     &array.3.into_iter(),
        // );
        // assert_eq!(array.into_iter().collect::<Vec<_>>(), vec);
    }
    //NullArray};

    // #[derive(Copy, Clone, Debug, Default, PartialEq)]
    // pub struct Unit;

    // pub type UnitArray = NullArray<Unit>;
    // impl StructArrayType for Unit {
    //     type Array = NullArray<Unit>;
    // }
    // impl ArrayType for Unit {
    //     type Array = StructArray<Unit, false>;
    // }
    // impl Array for NullArray<Unit> {
    //     type Validity = Self;
    //     fn validity(&self) -> &Self::Validity {
    //         self
    //     }
    //     fn len(&self) -> usize {
    //         self.len()
    //     }
    //     fn null_count(&self) -> usize {
    //         0 // this is questionable
    //     }
    //     fn is_null(&self, index: usize) -> bool {
    //         true
    //     }
    // }

    // #[derive(Array, Copy, Clone, Debug, Default, PartialEq)]
    // struct Foo {
    //     a: bool,
    //     b: Option<u32>,
    //     c: Bar,
    // }

    // #[derive(Array, Copy, Clone, Debug, Default, PartialEq)]
    // struct Bar {
    //     a: bool,
    // }

    // #[test]
    // fn derive() {
    //     let vec = vec![
    //         Foo {
    //             a: true,
    //             b: Some(1234),
    //             c: Bar { a: false },
    //         },
    //         Foo {
    //             a: false,
    //             b: None,
    //             c: Bar { a: false },
    //         },
    //         Foo {
    //             a: true,
    //             b: Some(42),
    //             c: Bar { a: true },
    //         },
    //     ];
    //     let array = vec.iter().copied().collect::<FooArray<false>>();
    //     assert_eq!(Array::len(&array), 3);
    //     assert_eq!(Array::len(&array.a), 3);
    //     assert_eq!(Array::len(&array.b), 3);
    //     // assert_eq!(Array::len(&array.d), 3);
    //     assert!(Array::all_valid(&array));
    //     assert!(!Array::is_empty(&array));
    //     assert_eq!(array.into_iter().collect::<Vec<_>>(), vec);
    //     assert_eq!(
    //         array.into_iter().map(|Foo { b, .. }| b).collect::<Vec<_>>(),
    //         vec![Some(1234), None, Some(42)]
    //     );
    //     let mut iter = array.a.into_iter();
    //     assert_eq!(iter.next(), Some(true));
    //     assert_eq!(iter.next(), Some(false));
    //     assert_eq!(iter.next(), Some(true));
    //     assert!(iter.next().is_none());
    //     assert_eq!(
    //         array.c.a,
    //         vec![false, false, true]
    //             .into_iter()
    //             .collect::<BooleanArray<false>>()
    //     );

    //     let vec = vec![
    //         Some(Foo {
    //             a: true,
    //             b: Some(1234),
    //             c: Bar { a: false },
    //         }),
    //         None,
    //         Some(Foo {
    //             a: false,
    //             b: None,
    //             c: Bar { a: false },
    //         }),
    //         Some(Foo {
    //             a: true,
    //             b: Some(42),
    //             c: Bar { a: true },
    //         }),
    //         None,
    //     ];
    //     let array = vec.iter().copied().collect::<FooArray<true>>();
    //     assert_eq!(array.into_iter().collect::<Vec<_>>(), vec);
    //     assert_eq!(Array::null_count(&array), 2);
    //     let mut iter = array.into_iter().map(|opt| opt.map(|Foo { b, .. }| b));
    //     assert_eq!(iter.size_hint(), (5, Some(5)));
    //     assert_eq!(iter.next(), Some(Some(Some(1234))));
    //     assert_eq!(iter.next(), Some(None));
    //     assert_eq!(iter.next(), Some(Some(None)));
    //     assert_eq!(iter.next(), Some(Some(Some(42))));
    //     assert_eq!(iter.next(), Some(None));
    //     assert!(iter.next().is_none());
    //     let mut iter = array.data().b.into_iter();
    //     assert_eq!(iter.size_hint(), (5, Some(5)));
    //     assert_eq!(iter.next(), Some(Some(1234)));
    //     assert_eq!(iter.next(), Some(None));
    //     assert_eq!(iter.next(), Some(None));
    //     assert_eq!(iter.next(), Some(Some(42)));
    //     assert_eq!(iter.next(), Some(None));
    //     assert!(iter.next().is_none());
    // }
}
