use super::{Array, ArrayType};
use crate::{
    buffer::{BufferType, VecBuffer},
    validity::Validity,
};

/// Struct array types.
pub trait StructArrayType: ArrayType {
    /// The array type that stores items of this struct. Note this differs from the `ArrayType` array because that wraps this array
    type Array<Buffer: BufferType>: Array;
}

pub struct StructArray<
    T: StructArrayType,
    const NULLABLE: bool = false,
    BitmapBuffer: BufferType = VecBuffer,
>(<<T as StructArrayType>::Array<BitmapBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer>)
where
    <T as StructArrayType>::Array<BitmapBuffer>: Validity<NULLABLE>;

impl<T: StructArrayType, const NULLABLE: bool, BitmapBuffer: BufferType> Array
    for StructArray<T, NULLABLE, BitmapBuffer>
where
    <T as StructArrayType>::Array<BitmapBuffer>: Validity<NULLABLE>,
{
    type Item = <<T as StructArrayType>::Array<BitmapBuffer> as Array>::Item;
}

impl<T: StructArrayType, U, const NULLABLE: bool, BitmapBuffer: BufferType> FromIterator<U>
    for StructArray<T, NULLABLE, BitmapBuffer>
where
    <T as StructArrayType>::Array<BitmapBuffer>: Validity<NULLABLE>,
    <<T as StructArrayType>::Array<BitmapBuffer> as Validity<NULLABLE>>::Storage<BitmapBuffer>:
        FromIterator<U>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter() {
        // Definition
        struct Foo {
            a: u32,
            b: Option<()>,
            c: (),
            d: Option<[u128; 2]>,
        }
        // These impls below can all be generated.
        impl ArrayType for Foo {
            type Array<Buffer: BufferType> = StructArray<Foo, false, Buffer>;
        }
        struct FooArray<Buffer: BufferType> {
            a: <u32 as ArrayType>::Array<Buffer>,
            b: <Option<()> as ArrayType>::Array<Buffer>,
            c: <() as ArrayType>::Array<Buffer>,
            d: <Option<[u128; 2]> as ArrayType>::Array<Buffer>,
        }
        impl<Buffer: BufferType> Array for FooArray<Buffer> {
            type Item = Foo;
        }
        impl<Buffer: BufferType> FromIterator<Foo> for FooArray<Buffer>
        where
            <u32 as ArrayType>::Array<Buffer>: Default + Extend<u32>,
            <Option<()> as ArrayType>::Array<Buffer>: Default + Extend<Option<()>>,
            <() as ArrayType>::Array<Buffer>: Default + Extend<()>,
            <Option<[u128; 2]> as ArrayType>::Array<Buffer>: Default + Extend<Option<[u128; 2]>>,
        {
            fn from_iter<T: IntoIterator<Item = Foo>>(iter: T) -> Self {
                let (a, (b, (c, d))) = iter
                    .into_iter()
                    .map(|Foo { a, b, c, d }| (a, (b, (c, d))))
                    .unzip();
                Self { a, b, c, d }
            }
        }
        impl StructArrayType for Foo {
            type Array<Buffer: BufferType> = FooArray<Buffer>;
        }

        // And then:
        let input = [
            Foo {
                a: 1,
                b: None,
                c: (),
                d: Some([1, 2]),
            },
            Foo {
                a: 2,
                b: Some(()),
                c: (),
                d: Some([3, 4]),
            },
            Foo {
                a: 3,
                b: None,
                c: (),
                d: None,
            },
            Foo {
                a: 4,
                b: None,
                c: (),
                d: None,
            },
        ];
        let array = input.into_iter().collect::<StructArray<Foo>>();
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
    }
}
