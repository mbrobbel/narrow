use super::{Array, ArrayType};
use crate::{buffer::Buffer, validity::Validity};

pub trait StructArrayType {
    type Array: Array;
}
pub struct StructArray<T, const NULLABLE: bool, BitmapBuffer = Vec<u8>>(
    <<T as StructArrayType>::Array as Validity<NULLABLE>>::Storage<BitmapBuffer>,
)
where
    T: StructArrayType,
    BitmapBuffer: Buffer<u8>,
    <T as StructArrayType>::Array: Validity<NULLABLE>;

struct Foo;
impl ArrayType for Foo {
    type Array<
        DataBuffer: crate::buffer::Buffer<Self::Primitive>,
        BitmapBuffer: crate::buffer::Buffer<u8>,
        OffsetElement: crate::offset::OffsetElement,
        OffsetBuffer: crate::buffer::Buffer<OffsetElement>,
    > = StructArray<Foo, false, BitmapBuffer>;
    type Primitive = u8;
    type RefItem<'a> = &'a Foo;
}
struct FooArray;
impl Array for FooArray {
    type Item = Foo;
}
impl StructArrayType for Foo {
    type Array = FooArray;
}

pub fn a() {
    let _x = StructArray::<Foo, false>(FooArray);
    let _y: <Foo as ArrayType>::Array<Vec<u8>, Vec<u8>, i32, Vec<i32>> = StructArray(FooArray);
    let _z: <Foo as StructArrayType>::Array = FooArray;
}
