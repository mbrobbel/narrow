//! Array for product types.

use super::{Array, ArrayType};
use crate::{
    bitmap::{Bitmap, BitmapRef, BitmapRefMut, ValidityBitmap},
    buffer::{BufferType, VecBuffer},
    nullable::Nullable,
    validity::{Nullability, Validity},
    Length,
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
    const NULLABLE: bool = false,
    Buffer: BufferType = VecBuffer,
>(pub <<T as StructArrayType>::Array<Buffer> as Validity<NULLABLE>>::Storage<Buffer>)
where
    <T as StructArrayType>::Array<Buffer>: Validity<NULLABLE>;

impl<T: StructArrayType, const NULLABLE: bool, Buffer: BufferType> StructArray<T, NULLABLE, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: Validity<NULLABLE>,
    for<'a> &'a Self: IntoIterator,
{
    /// Returns an iterator over the items in this [`StructArray`].
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl<T: StructArrayType, const NULLABLE: bool, Buffer: BufferType> Array
    for StructArray<T, NULLABLE, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: Validity<NULLABLE>,
    T: Nullability<NULLABLE>,
{
    type Item = <T as Nullability<NULLABLE>>::Item;
}

impl<T: StructArrayType, const NULLABLE: bool, Buffer: BufferType> Clone
    for StructArray<T, NULLABLE, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: Validity<NULLABLE>,
    <<T as StructArrayType>::Array<Buffer> as Validity<NULLABLE>>::Storage<Buffer>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: StructArrayType, const NULLABLE: bool, Buffer: BufferType> Default
    for StructArray<T, NULLABLE, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: Validity<NULLABLE>,
    <<T as StructArrayType>::Array<Buffer> as Validity<NULLABLE>>::Storage<Buffer>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: StructArrayType, Buffer: BufferType> From<StructArray<T, false, Buffer>>
    for StructArray<T, true, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: Length,
    Bitmap<Buffer>: FromIterator<bool>,
{
    fn from(value: StructArray<T, false, Buffer>) -> Self {
        Self(Nullable::from(value.0))
    }
}

impl<T: StructArrayType, U, const NULLABLE: bool, Buffer: BufferType> FromIterator<U>
    for StructArray<T, NULLABLE, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: Validity<NULLABLE>,
    <<T as StructArrayType>::Array<Buffer> as Validity<NULLABLE>>::Storage<Buffer>: FromIterator<U>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: StructArrayType, U, Buffer: BufferType> Extend<U> for StructArray<T, false, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: Extend<U>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: StructArrayType, U, Buffer: BufferType> Extend<Option<U>> for StructArray<T, true, Buffer>
where
    Nullable<<T as StructArrayType>::Array<Buffer>, Buffer>: Extend<Option<U>>,
{
    fn extend<I: IntoIterator<Item = Option<U>>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T: StructArrayType, const NULLABLE: bool, Buffer: BufferType> IntoIterator
    for StructArray<T, NULLABLE, Buffer>
where
    T: Nullability<NULLABLE>,
    <T as StructArrayType>::Array<Buffer>: Validity<NULLABLE>,
    <<T as StructArrayType>::Array<Buffer> as Validity<NULLABLE>>::Storage<Buffer>: IntoIterator,
{
    type Item = <<<T as StructArrayType>::Array<Buffer> as Validity<NULLABLE>>::Storage<Buffer> as
        IntoIterator>::Item;
    type IntoIter = <<<T as StructArrayType>::Array<Buffer> as Validity<NULLABLE>>::Storage<Buffer> as
        IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: StructArrayType, const NULLABLE: bool, Buffer: BufferType> IntoIterator
    for &'a StructArray<T, NULLABLE, Buffer>
where
    T: Nullability<NULLABLE>,
    <T as StructArrayType>::Array<Buffer>: Validity<NULLABLE>,
    &'a <<T as StructArrayType>::Array<Buffer> as Validity<NULLABLE>>::Storage<Buffer>:
        IntoIterator,
{
    type Item = <&'a <<T as StructArrayType>::Array<Buffer> as Validity<NULLABLE>>::Storage<Buffer> as
        IntoIterator>::Item;
    type IntoIter = <&'a <<T as StructArrayType>::Array<Buffer> as Validity<NULLABLE>>::Storage<
        Buffer,
    > as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: StructArrayType, const NULLABLE: bool, Buffer: BufferType> Length
    for StructArray<T, NULLABLE, Buffer>
where
    <T as StructArrayType>::Array<Buffer>: Validity<NULLABLE>,
    <<T as StructArrayType>::Array<Buffer> as Validity<NULLABLE>>::Storage<Buffer>: Length,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: StructArrayType, Buffer: BufferType> BitmapRef for StructArray<T, true, Buffer> {
    type Buffer = Buffer;

    fn bitmap_ref(&self) -> &Bitmap<Self::Buffer> {
        self.0.bitmap_ref()
    }
}

impl<T: StructArrayType, Buffer: BufferType> BitmapRefMut for StructArray<T, true, Buffer> {
    fn bitmap_ref_mut(&mut self) -> &mut Bitmap<Self::Buffer> {
        self.0.bitmap_ref_mut()
    }
}

impl<T: StructArrayType, Buffer: BufferType> ValidityBitmap for StructArray<T, true, Buffer> {}

#[cfg(test)]
mod tests {
    use crate::{
        array::{union, UnionType},
        offset::{self, OffsetElement},
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
        type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
            StructArray<Foo<'a>, false, Buffer>;
    }
    impl<'a> ArrayType<Foo<'a>> for Option<Foo<'a>> {
        type Array<Buffer: BufferType, OffsetItem: OffsetElement, UnionLayout: UnionType> =
            StructArray<Foo<'a>, true, Buffer>;
    }

    struct FooArray<'a, Buffer: BufferType> {
        a: <u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA>,
        b: <Option<()> as ArrayType<()>>::Array<Buffer, offset::NA, union::NA>,
        c: <() as ArrayType<()>>::Array<Buffer, offset::NA, union::NA>,
        d: <Option<[u64; 2]> as ArrayType<[u64; 2]>>::Array<Buffer, offset::NA, union::NA>,
        e: <bool as ArrayType<bool>>::Array<Buffer, offset::NA, union::NA>,
        f: <&'a [u8] as ArrayType<&'a [u8]>>::Array<Buffer, offset::NA, union::NA>,
        g: <String as ArrayType<String>>::Array<Buffer, offset::NA, union::NA>,
    }

    impl<'a, Buffer: BufferType> Default for FooArray<'a, Buffer>
    where
        <u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA>: Default,
        <Option<()> as ArrayType<()>>::Array<Buffer, offset::NA, union::NA>: Default,
        <() as ArrayType<()>>::Array<Buffer, offset::NA, union::NA>: Default,
        <Option<[u64; 2]> as ArrayType<[u64; 2]>>::Array<Buffer, offset::NA, union::NA>: Default,
        <bool as ArrayType<bool>>::Array<Buffer, offset::NA, union::NA>: Default,
        <&'a [u8] as ArrayType<&'a [u8]>>::Array<Buffer, offset::NA, union::NA>: Default,
        <String as ArrayType<String>>::Array<Buffer, offset::NA, union::NA>: Default,
    {
        fn default() -> Self {
            Self {
                a: <u32 as ArrayType<u32>>::Array::<Buffer, offset::NA, union::NA>::default(),
                b: <Option<()> as ArrayType<()>>::Array::<Buffer, offset::NA, union::NA>::default(),
                c: <() as ArrayType<()>>::Array::<Buffer, offset::NA, union::NA>::default(),
                d: <Option<[u64; 2]> as ArrayType<[u64; 2]>>::Array::<Buffer, offset::NA, union::NA>::default(
                ),
                e: <bool as ArrayType<bool>>::Array::<Buffer, offset::NA, union::NA>::default(),
                f: <&'a [u8] as ArrayType<&'a [u8]>>::Array::<Buffer, offset::NA, union::NA>::default(),
                g: <String as ArrayType<String>>::Array::<Buffer, offset::NA, union::NA>::default(),
            }
        }
    }

    impl<'a, Buffer: BufferType> Extend<Foo<'a>> for FooArray<'a, Buffer>
    where
        <u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA>: Extend<u32>,
        <Option<()> as ArrayType<()>>::Array<Buffer, offset::NA, union::NA>: Extend<Option<()>>,
        <() as ArrayType<()>>::Array<Buffer, offset::NA, union::NA>: Extend<()>,
        <Option<[u64; 2]> as ArrayType<[u64; 2]>>::Array<Buffer, offset::NA, union::NA>:
            Extend<Option<[u64; 2]>>,
        <bool as ArrayType<bool>>::Array<Buffer, offset::NA, union::NA>: Extend<bool>,
        <&'a [u8] as ArrayType<&'a [u8]>>::Array<Buffer, offset::NA, union::NA>: Extend<&'a [u8]>,
        <String as ArrayType<String>>::Array<Buffer, offset::NA, union::NA>: Extend<String>,
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
        <u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA>: Default + Extend<u32>,
        <Option<()> as ArrayType<()>>::Array<Buffer, offset::NA, union::NA>:
            Default + Extend<Option<()>>,
        <() as ArrayType<()>>::Array<Buffer, offset::NA, union::NA>: Default + Extend<()>,
        <Option<[u64; 2]> as ArrayType<[u64; 2]>>::Array<Buffer, offset::NA, union::NA>:
            Default + Extend<Option<[u64; 2]>>,
        <bool as ArrayType<bool>>::Array<Buffer, offset::NA, union::NA>: Default + Extend<bool>,
        <&'a [u8] as ArrayType<&'a [u8]>>::Array<Buffer, offset::NA, union::NA>:
            Default + Extend<&'a [u8]>,
        <String as ArrayType<String>>::Array<Buffer, offset::NA, union::NA>:
            Default + Extend<String>,
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

    impl<'a, Buffer: BufferType> Length for FooArray<'a, Buffer>
    where
        <u32 as ArrayType<u32>>::Array<Buffer, offset::NA, union::NA>: Length,
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
            array.0.g.0 .0.data.into_iter().collect::<Vec<_>>(),
            &[97, 115, 100, 102] // a s d f
        );
        assert_eq!(
            array.0.g.0 .0.offsets.into_iter().collect::<Vec<_>>(),
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
            .collect::<StructArray<Foo, true>>();
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

        let mut foo_bar = StructArray::<FooBar, false>::default();
        foo_bar.extend(std::iter::once(FooBar(None)));
        foo_bar.extend(std::iter::once(FooBar(Some(vec![None]))));
        let mut foo_bar_nullable = StructArray::<FooBar, true>::default();
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
            .collect::<StructArray<Unit, true>>();
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
            .collect::<StructArray<Unnamed, true>>();
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
            .collect::<StructArray<Named, true>>();
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
