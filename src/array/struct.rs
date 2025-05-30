//! Array for product types.

use super::{Array, ArrayType};
use crate::{
    Length,
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{BufferType, VecBuffer},
    nullability::{NonNullable, Nullability, Nullable},
    validity::Validity,
};

/// Struct array types.
pub trait StructArrayType: ArrayType<Self> {
    /// The array type that stores items of this struct. Note this differs from
    /// the [`ArrayType`] array because that wraps this array. Also note that this
    /// has no [`Array`] bound.
    // TODO(mbrobbe): add offset and union generics
    type Array<Buffer: BufferType>; // into<fields> this then requires all arraytype impls to provide a field
}

/// Array for product types.
pub struct StructArray<
    T: StructArrayType,
    Nullable: Nullability = NonNullable,
    Buffer: BufferType = VecBuffer,
>(pub Nullable::Collection<<T as StructArrayType>::Array<Buffer>, Buffer>);

impl<T: StructArrayType, Nullable: Nullability, Buffer: BufferType> StructArray<T, Nullable, Buffer>
where
    for<'a> &'a Self: IntoIterator,
{
    /// Returns an iterator over the items in this [`StructArray`].
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl<T: StructArrayType, Nullable: Nullability, Buffer: BufferType> Array
    for StructArray<T, Nullable, Buffer>
{
    type Item = Nullable::Item<T>;
}

impl<T: StructArrayType, Nullable: Nullability, Buffer: BufferType> Clone
    for StructArray<T, Nullable, Buffer>
where
    Nullable::Collection<<T as StructArrayType>::Array<Buffer>, Buffer>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: StructArrayType, Nullable: Nullability, Buffer: BufferType> Default
    for StructArray<T, Nullable, Buffer>
where
    Nullable::Collection<<T as StructArrayType>::Array<Buffer>, Buffer>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: StructArrayType, Buffer: BufferType> From<StructArray<T, NonNullable, Buffer>>
    for StructArray<T, Nullable, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: Length,
    Bitmap<Buffer>: FromIterator<bool>,
{
    fn from(value: StructArray<T, NonNullable, Buffer>) -> Self {
        Self(Validity::from(value.0))
    }
}

impl<T: StructArrayType, U, Nullable: Nullability, Buffer: BufferType> FromIterator<U>
    for StructArray<T, Nullable, Buffer>
where
    Nullable::Collection<<T as StructArrayType>::Array<Buffer>, Buffer>: FromIterator<U>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: StructArrayType, U, Buffer: BufferType> Extend<U> for StructArray<T, NonNullable, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: Extend<U>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: StructArrayType, U, Buffer: BufferType> Extend<Option<U>>
    for StructArray<T, Nullable, Buffer>
where
    Validity<<T as StructArrayType>::Array<Buffer>, Buffer>: Extend<Option<U>>,
{
    fn extend<I: IntoIterator<Item = Option<U>>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: StructArrayType, Nullable: Nullability, Buffer: BufferType> IntoIterator
    for StructArray<T, Nullable, Buffer>
where
    Nullable::Collection<<T as StructArrayType>::Array<Buffer>, Buffer>: IntoIterator,
{
    type Item =
        <Nullable::Collection<<T as StructArrayType>::Array<Buffer>, Buffer> as IntoIterator>::Item;
    type IntoIter = <Nullable::Collection<<T as StructArrayType>::Array<Buffer>, Buffer> as
        IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: StructArrayType, Nullable: Nullability, Buffer: BufferType> IntoIterator
    for &'a StructArray<T, Nullable, Buffer>
where
    &'a Nullable::Collection<<T as StructArrayType>::Array<Buffer>, Buffer>: IntoIterator,
{
    type Item = <&'a Nullable::Collection<<T as StructArrayType>::Array<Buffer>, Buffer> as
        IntoIterator>::Item;
    type IntoIter = <&'a Nullable::Collection<<T as StructArrayType>::Array<Buffer>, Buffer> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: StructArrayType, Nullable: Nullability, Buffer: BufferType> Length
    for StructArray<T, Nullable, Buffer>
where
    Nullable::Collection<<T as StructArrayType>::Array<Buffer>, Buffer>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: StructArrayType, Buffer: BufferType> BitmapRef for StructArray<T, Nullable, Buffer> {
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<T: StructArrayType, Buffer: BufferType> BitmapRefMut for StructArray<T, Nullable, Buffer> {
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<T: StructArrayType, Buffer: BufferType> ValidityBitmap for StructArray<T, Nullable, Buffer> {}

#[cfg(test)]
mod tests {
    use crate::{
        array::{ArrayTypeOf, OptionArrayTypeOf, UnionType},
        offset::Offset,
    };

    use super::*;

    // Definition
    #[derive(Default)]
    struct Foo<'a> {
        a: u32,
        b: Option<()>,
        c: (),
        d: Option<[u64; 2]>,
        e: bool,
        f: &'a [u8],
        g: String,
    }
    // These impls below can all be generated.
    impl<'a> ArrayType<Self> for Foo<'a> {
        type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
            StructArray<Foo<'a>, NonNullable, Buffer>;
    }
    impl<'a> ArrayType<Foo<'a>> for Option<Foo<'a>> {
        type Array<Buffer: BufferType, OffsetItem: Offset, UnionLayout: UnionType> =
            StructArray<Foo<'a>, Nullable, Buffer>;
    }

    struct FooArray<'a, Buffer: BufferType> {
        a: ArrayTypeOf<u32, Buffer>,
        b: OptionArrayTypeOf<(), Buffer>,
        c: ArrayTypeOf<(), Buffer>,
        d: OptionArrayTypeOf<[u64; 2], Buffer>,
        e: ArrayTypeOf<bool, Buffer>,
        f: ArrayTypeOf<&'a [u8], Buffer>,
        g: ArrayTypeOf<String, Buffer>,
    }

    impl<'a, Buffer: BufferType> Default for FooArray<'a, Buffer>
    where
        ArrayTypeOf<u32, Buffer>: Default,
        OptionArrayTypeOf<(), Buffer>: Default,
        ArrayTypeOf<(), Buffer>: Default,
        OptionArrayTypeOf<[u64; 2], Buffer>: Default,
        ArrayTypeOf<bool, Buffer>: Default,
        ArrayTypeOf<&'a [u8], Buffer>: Default,
        ArrayTypeOf<String, Buffer>: Default,
    {
        fn default() -> Self {
            Self {
                a: <ArrayTypeOf<u32, Buffer>>::default(),
                b: <OptionArrayTypeOf<(), Buffer>>::default(),
                c: <ArrayTypeOf<(), Buffer>>::default(),
                d: <OptionArrayTypeOf<[u64; 2], Buffer>>::default(),
                e: <ArrayTypeOf<bool, Buffer>>::default(),
                f: <ArrayTypeOf<&'a [u8], Buffer>>::default(),
                g: <ArrayTypeOf<String, Buffer>>::default(),
            }
        }
    }

    impl<'a, Buffer: BufferType> Extend<Foo<'a>> for FooArray<'a, Buffer>
    where
        ArrayTypeOf<u32, Buffer>: Extend<u32>,
        OptionArrayTypeOf<(), Buffer>: Extend<Option<()>>,
        ArrayTypeOf<(), Buffer>: Extend<()>,
        OptionArrayTypeOf<[u64; 2], Buffer>: Extend<Option<[u64; 2]>>,
        ArrayTypeOf<bool, Buffer>: Extend<bool>,
        ArrayTypeOf<&'a [u8], Buffer>: Extend<&'a [u8]>,
        ArrayTypeOf<String, Buffer>: Extend<String>,
    {
        fn extend<I: IntoIterator<Item = Foo<'a>>>(&mut self, iter: I) {
            iter.into_iter().for_each(
                |Foo {
                     a,
                     b,
                     c,
                     d,
                     e,
                     f,
                     g,
                 }| {
                    self.a.extend(std::iter::once(a));
                    self.b.extend(std::iter::once(b));
                    self.c.extend(std::iter::once(c));
                    self.d.extend(std::iter::once(d));
                    self.e.extend(std::iter::once(e));
                    self.f.extend(std::iter::once(f));
                    self.g.extend(std::iter::once(g));
                },
            );
        }
    }

    impl<'a, Buffer: BufferType> FromIterator<Foo<'a>> for FooArray<'a, Buffer>
    where
        ArrayTypeOf<u32, Buffer>: Default + Extend<u32>,
        OptionArrayTypeOf<(), Buffer>: Default + Extend<Option<()>>,
        ArrayTypeOf<(), Buffer>: Default + Extend<()>,
        OptionArrayTypeOf<[u64; 2], Buffer>: Default + Extend<Option<[u64; 2]>>,
        ArrayTypeOf<bool, Buffer>: Default + Extend<bool>,
        ArrayTypeOf<&'a [u8], Buffer>: Default + Extend<&'a [u8]>,
        ArrayTypeOf<String, Buffer>: Default + Extend<String>,
    {
        #[allow(clippy::many_single_char_names)]
        fn from_iter<T: IntoIterator<Item = Foo<'a>>>(iter: T) -> Self {
            let (a, (b, (c, (d, (e, (f, g)))))) = iter
                .into_iter()
                .map(
                    |Foo {
                         a,
                         b,
                         c,
                         d,
                         e,
                         f,
                         g,
                     }| (a, (b, (c, (d, (e, (f, g)))))),
                )
                .unzip();
            Self {
                a,
                b,
                c,
                d,
                e,
                f,
                g,
            }
        }
    }
    impl<'a> StructArrayType for Foo<'a> {
        type Array<Buffer: BufferType> = FooArray<'a, Buffer>;
    }

    impl<Buffer: BufferType> Length for FooArray<'_, Buffer>
    where
        ArrayTypeOf<u32, Buffer>: Length,
    {
        fn len(&self) -> usize {
            self.a.len()
        }
    }

    #[test]
    fn from_iter() {
        let input = [
            Foo {
                a: 1,
                b: None,
                c: (),
                d: Some([1, 2]),
                e: false,
                f: &[1],
                g: "a".to_owned(),
            },
            Foo {
                a: 2,
                b: Some(()),
                c: (),
                d: Some([3, 4]),
                e: true,
                f: &[2, 3],
                g: "s".to_owned(),
            },
            Foo {
                a: 3,
                b: None,
                c: (),
                d: None,
                e: true,
                f: &[4],
                g: "d".to_owned(),
            },
            Foo {
                a: 4,
                b: None,
                c: (),
                d: None,
                e: true,
                f: &[],
                g: "f".to_owned(),
            },
        ];
        let array = input.into_iter().collect::<StructArray<Foo>>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.0.a.into_iter().collect::<Vec<_>>(), &[1, 2, 3, 4]);
        assert_eq!(
            array.0.b.into_iter().collect::<Vec<_>>(),
            &[None, Some(()), None, None]
        );
        assert_eq!(array.0.c.into_iter().collect::<Vec<_>>(), &[(), (), (), ()]);
        assert_eq!(
            array.0.d.into_iter().collect::<Vec<_>>(),
            &[Some([1, 2]), Some([3, 4]), None, None]
        );
        assert_eq!(
            array.0.e.into_iter().collect::<Vec<_>>(),
            &[false, true, true, true]
        );
        assert_eq!(
            array.0.f.0.data.into_iter().collect::<Vec<_>>(),
            &[1, 2, 3, 4]
        );
        assert_eq!(
            array.0.f.0.offsets.into_iter().collect::<Vec<_>>(),
            &[0, 1, 3, 4, 4]
        );
        assert_eq!(
            array.0.g.0.0.data.into_iter().collect::<Vec<_>>(),
            &[97, 115, 100, 102] // a s d f
        );
        assert_eq!(
            array.0.g.0.0.offsets.into_iter().collect::<Vec<_>>(),
            &[0, 1, 2, 3, 4]
        );

        let input_nullable = [
            None,
            Some(Foo {
                a: 1,
                b: None,
                c: (),
                d: Some([1, 2]),
                e: false,
                f: &[1],
                g: "a".to_owned(),
            }),
        ];
        let array_nullable = input_nullable
            .into_iter()
            .collect::<StructArray<Foo, Nullable>>();
        assert_eq!(array_nullable.len(), 2);
        assert_eq!(array_nullable.is_null(0), Some(true));
        assert_eq!(array_nullable.is_valid(1), Some(true));
        assert_eq!(array_nullable.is_valid(2), None);
    }

    #[cfg(feature = "derive")]
    #[test]
    fn derive() {
        #[derive(crate::ArrayType, Copy, Clone, Default)]
        struct Unit;

        #[derive(crate::ArrayType)]
        struct Foo(Option<Bar<u32>>);

        #[derive(crate::ArrayType)]
        struct Bar<T>(T);

        #[derive(crate::ArrayType, Default)]
        struct FooBar(Option<Vec<Option<u32>>>);

        let mut foo_bar = StructArray::<FooBar, NonNullable>::default();
        foo_bar.extend(std::iter::once(FooBar(None)));
        foo_bar.extend(std::iter::once(FooBar(Some(vec![None]))));
        let mut foo_bar_nullable = StructArray::<FooBar, Nullable>::default();
        foo_bar_nullable.extend(std::iter::once(Some(FooBar(None))));
        foo_bar_nullable.extend(std::iter::once(None));
    }

    #[cfg(feature = "derive")]
    #[test]
    fn into_iter() {
        #[derive(crate::ArrayType, Copy, Clone, Debug, Default, PartialEq)]
        struct Unit;

        #[derive(crate::ArrayType, Copy, Clone, Debug, Default, PartialEq)]
        struct Unnamed(u8, Option<u16>, u32, u64);

        #[derive(crate::ArrayType, Clone, Debug, Default, PartialEq)]
        struct Named {
            a: u8,
            b: bool,
            c: Option<u16>,
            d: Option<bool>,
            e: String,
            f: Option<String>,
        }

        let unit_input = [Unit; 3];
        let unit_array = unit_input.into_iter().collect::<StructArray<Unit>>();
        assert_eq!(unit_array.len(), unit_input.len());
        let unit_output = unit_array.into_iter().collect::<Vec<_>>();
        assert_eq!(unit_output, unit_input);
        let unit_input_nullable = unit_input.map(Option::Some);
        let unit_array_nullable = unit_input_nullable
            .into_iter()
            .collect::<StructArray<Unit, Nullable>>();
        assert_eq!(unit_array_nullable.len(), unit_input_nullable.len());
        let unit_output_nullable = unit_array_nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(unit_output_nullable, unit_input_nullable);

        let unnamed_input = [Unnamed(1, Some(2), 3, 4); 3];
        let unnamed_array = unnamed_input.into_iter().collect::<StructArray<Unnamed>>();
        assert_eq!(unnamed_array.len(), unnamed_input.len());
        let unnamed_output = unnamed_array.into_iter().collect::<Vec<_>>();
        assert_eq!(unnamed_output, unnamed_input);
        let unnamed_input_nullable = unnamed_input.map(Option::Some);
        let unnamed_array_nullable = unnamed_input_nullable
            .into_iter()
            .collect::<StructArray<Unnamed, Nullable>>();
        assert_eq!(unnamed_array_nullable.len(), unnamed_input_nullable.len());
        let unnamed_output_nullable = unnamed_array_nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(unnamed_output_nullable, unnamed_input_nullable);

        let named_input = [Named {
            a: 1,
            b: false,
            c: Some(3),
            d: Some(true),
            e: "hello".to_owned(),
            f: None,
        }];
        let named_array = named_input
            .clone()
            .into_iter()
            .collect::<StructArray<Named>>();
        assert_eq!(named_array.len(), named_input.len());
        let named_output = named_array.into_iter().collect::<Vec<_>>();
        assert_eq!(named_output, named_input);
        let named_input_nullable = named_input
            .into_iter()
            .map(Option::Some)
            .collect::<Vec<_>>();
        let named_array_nullable = named_input_nullable
            .clone()
            .into_iter()
            .collect::<StructArray<Named, Nullable>>();
        assert_eq!(named_array_nullable.len(), named_input_nullable.len());
        let named_output_nullable = named_array_nullable.into_iter().collect::<Vec<_>>();
        assert_eq!(named_output_nullable, named_input_nullable);
    }

    #[cfg(feature = "derive")]
    #[test]
    fn nested_option_derived() {
        use std::collections::VecDeque;

        {
            #[derive(crate::ArrayType, Clone, Debug, PartialEq)]
            struct Foo(Vec<Option<String>>);

            let input = [Foo(vec![None]), Foo(vec![Some("hello".to_owned())])];
            let array = input.clone().into_iter().collect::<StructArray<Foo>>();
            assert_eq!(array.len(), 2);
            let output = array.into_iter().collect::<Vec<_>>();
            assert_eq!(input.as_slice(), output);
        };

        {
            #[derive(crate::ArrayType, Clone, Debug, PartialEq)]
            struct Foo([Option<String>; 1]);

            let input = [Foo([None]), Foo([Some("hello".to_owned())])];
            let array = input.clone().into_iter().collect::<StructArray<Foo>>();
            assert_eq!(array.len(), 2);
            let output = array.into_iter().collect::<Vec<_>>();
            assert_eq!(input.as_slice(), output);
        };

        {
            #[derive(crate::ArrayType, Clone, Debug, PartialEq)]
            struct Foo(VecDeque<Option<String>>);

            let input = [
                Foo(VecDeque::from_iter([None])),
                Foo(VecDeque::from_iter([Some("hello".to_owned())])),
            ];
            let array = input.clone().into_iter().collect::<StructArray<Foo>>();
            assert_eq!(array.len(), 2);
            // TODO(mbrobbel): add support to offset iterator for VecDeque
            // OR fix bound + convert via proc macro
            // let output = array.into_iter().collect::<Vec<_>>();
            // assert_eq!(input.as_slice(), output);
        };
    }
}
