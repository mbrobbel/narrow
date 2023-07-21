//! Sequences of values with known length all having the same type.

// use self::{
//     boolean::BooleanArray,
//     fixed_size_primitive::{
//         Float32Array, Float64Array, Int16Array, Int32Array, Int64Array, Int8Array, Uint16Array,
//         Uint32Array, Uint64Array, Uint8Array,
//     },
//     null::NullArray,
// };
// use crate::{buffer::Buffer, offset, Primitive};

// pub mod boolean;
// pub mod fixed_size_primitive;
// pub mod null;
// pub mod run_end_encoded;
// pub mod string;
// pub mod variable_size_binary;

use crate::{
    buffer::{self, VecBuffer},
    validity::Validity,
};

/// Trait to construct arrays.
pub trait ArrayConstructor: ArrayType {
    type Array<'a, const NULLABLE: bool, Buffer: buffer::Buffer + buffer::Buffer<NULLABLE>>;
}

/// Used to get the concrete array impl of something. Uses the ArrayConstructor trait.
pub trait ArrayType {
    type Array<'a, Buffer: buffer::Buffer + buffer::Buffer<true>>;
}

// /// Implemented by arrays
pub trait Array {}

pub struct BooleanArray<'a, const NULLABLE: bool, Buffer: buffer::Buffer + buffer::Buffer<NULLABLE>>(
    <<Buffer as buffer::Buffer<NULLABLE>>::Container<'a, u8> as Validity<NULLABLE>>::Storage<
        'a,
        Buffer,
    >,
);

impl<'a, const NULLABLE: bool, Buffer: buffer::Buffer + buffer::Buffer<NULLABLE>> Array
    for BooleanArray<'a, NULLABLE, Buffer>
where
    <Buffer as buffer::Buffer>::Container<'a, u8>: Validity<NULLABLE>,
{
}

impl ArrayConstructor for bool {
    type Array<'a, const NULLABLE: bool, Buffer: buffer::Buffer + buffer::Buffer<NULLABLE>> =
        BooleanArray<'a, NULLABLE, Buffer>;
}
impl ArrayType for bool {
    type Array<'a, Buffer: buffer::Buffer + buffer::Buffer<true>> =
        <bool as ArrayConstructor>::Array<'a, false, Buffer>;
}

impl<T> ArrayType for Option<T>
where
    T: ArrayConstructor,
{
    type Array<'a, Buffer: buffer::Buffer + buffer::Buffer<true>> =
        <T as ArrayConstructor>::Array<'a, true, Buffer>;
}

pub fn a() {
    let x: <bool as ArrayType>::Array<'_, VecBuffer> = BooleanArray::<false, VecBuffer>(vec![1u8]);
    // let x: <bool as ArrayType>::Array = BooleanArray::<true>;
    let y: <Option<bool> as ArrayType>::Array<'_, VecBuffer> =
        BooleanArray::<true, VecBuffer>(vec![1u8]);
    // let y: <Option<bool> as ArrayType>::Array = BooleanArray::<false>;
}

// /// implemented by data structures that are arrow arrays
// pub trait Array {
//     /// Array constructor
//     type Array<const NULLABLE: bool>: ArrayX;
// }

// pub trait ArrayX {
//     type Item;
// }

// pub trait ArrayType {
//     /// The [Array] type that stores values of this type.
//     type Array: ArrayX;

//     // /// Storage type in the data buffer. (This is weird for null arrays).
//     // type Primitive: Primitive;

//     //A reference type for this type that is used when borrowing data from the
//     // / array.
//     // type RefItem<'a>;
// }

// pub struct BooleanArray<const NULLABLE: bool>;

// impl<const NULLABLE: bool> ArrayX for BooleanArray<NULLABLE> {
//     type Item = bool;
// }

// impl Array for bool {
//     type Array<const NULLABLE: bool> = BooleanArray<NULLABLE>;
// }

// impl ArrayType for bool {
//     type Array = <Self as Array>::Array<false>;
// }

// impl<T> ArrayType for Option<T>
// where
//     T: ArrayType,
//     <T as ArrayType>::Array: ArrayX,
// {
//     type Array = <<T as ArrayType>::Array as Array>::Array<true>;
// }

// fn a() {
//     let x: <bool as ArrayType>::Array;
//     let y: <Option<bool> as ArrayType>::Array;
// }

// <u64 as ArrayType>::Array<Buffer, _>
// <Option<u64> as ArrayType>::Array<Buffer, _>

// impl<T> ArrayType for Option<T>
// where
//     T: ArrayType,
// {
//     type Array<
//         const NULLABLE: bool,
//         Buffer: buffer::Buffer,
//         OffsetElement: offset::OffsetElement,
//         // // The buffer type for data
//         // DataBuffer: Buffer<Self::Primitive>,
//         // // The buffer type for the bitmap, when nullable
//         // BitmapBuffer: Buffer<u8>,
//         // OffsetElement: offset::OffsetElement,
//         // OffsetBuffer: Buffer<OffsetElement>,
//     > = <T as ArrayType>::Array<true, Buffer, OffsetElement>;

//     type RefItem<'a> = ();
// }

// macro_rules! impl_array_type {
//     ($ty:ty, $prim:ty, $array:ty, $item:ty) => {
//         impl ArrayType for $ty {
//             type Array<
//                 DataBuffer: Buffer<Self::Primitive>,
//                 BitmapBuffer: Buffer<u8>,
//                 OffsetElement: offset::OffsetElement,
//                 OffsetBuffer: Buffer<OffsetElement>,
//             > = $array;
//             type Primitive = $prim;
//             type RefItem<'a> = $item;
//         }
//     };
//     ($ty:ty, $array:ty) => {
//         impl_array_type!($ty, $ty, $array, $ty);
//     };
// }

// impl_array_type!((), u8, NullArray<(), false>, ());
// impl_array_type!(Option<()>, u8, NullArray<(), true, BitmapBuffer>, Option<&'a()>);

// impl_array_type!(bool, u8, BooleanArray<false, DataBuffer>, bool);
// impl_array_type!(Option<bool>, u8, BooleanArray<true, DataBuffer, BitmapBuffer>, Option<&'a bool>);

// impl_array_type!(i8, Int8Array<false, DataBuffer>);
// impl_array_type!(Option<i8>, i8, Int8Array<true, DataBuffer, BitmapBuffer>, Option<&'a i8>);
// impl_array_type!(i16, Int16Array<false, DataBuffer>);
// impl_array_type!(Option<i16>, i16, Int16Array<true, DataBuffer, BitmapBuffer>, Option<&'a i16>);
// impl_array_type!(i32, Int32Array<false, DataBuffer>);
// impl_array_type!(Option<i32>, i32, Int32Array<true, DataBuffer, BitmapBuffer>, Option<&'a i32>);
// impl_array_type!(i64, Int64Array<false, DataBuffer>);
// impl_array_type!(Option<i64>, i64, Int64Array<true, DataBuffer, BitmapBuffer>, Option<&'a i64>);

// impl_array_type!(u8, Uint8Array<false, DataBuffer>);
// impl_array_type!(Option<u8>, u8, Uint8Array<true, DataBuffer, BitmapBuffer>, Option<&'a u8>);
// impl_array_type!(u16, Uint16Array<false, DataBuffer>);
// impl_array_type!(Option<u16>, u16, Uint16Array<true, DataBuffer, BitmapBuffer>, Option<&'a u16>);
// impl_array_type!(u32, Uint32Array<false, DataBuffer>);
// impl_array_type!(Option<u32>, u32, Uint32Array<true, DataBuffer, BitmapBuffer>, Option<&'a u32>);
// impl_array_type!(u64, Uint64Array<false, DataBuffer>);
// impl_array_type!(Option<u64>, u64, Uint64Array<true, DataBuffer, BitmapBuffer>, Option<&'a u64>);

// impl_array_type!(f32, Float32Array<false, DataBuffer>);
// impl_array_type!(Option<f32>, f32, Float32Array<true, DataBuffer, BitmapBuffer>, Option<&'a f32>);
// impl_array_type!(f64, Float64Array<false, DataBuffer>);
// impl_array_type!(Option<f64>, f64, Float64Array<true, DataBuffer, BitmapBuffer>, Option<&'a f64>);
