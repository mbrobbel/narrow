//! # narrow
//!
//! An experimental (work-in-progress) implementation of [Apache Arrow](https://arrow.apache.org).

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/mbrobbel/narrow/main/narrow.svg",
    html_favicon_url = "https://raw.githubusercontent.com/mbrobbel/narrow/main/narrow.svg"
)]
// #![deny(warnings)]

mod fixed_size;
pub use self::fixed_size::FixedSize;

mod length;
pub use self::length::Length;

pub mod buffer;

pub mod bitmap;

pub(crate) mod nullable;
// // // // pub(crate) mod offset;
pub(crate) mod validity;

pub mod array;

// // Re-export `narrow_derive` macros when the `derive` feature is enabled.
// #[cfg(feature = "derive")]
// pub use narrow_derive::Array;

// trait Buffer {
//     type Container<'a, T>: std::borrow::Borrow<[T]>;
// }
// struct BufferA {}
// impl Buffer for BufferA {
//     type Container<'a, T> = Vec<T>;
// }

// struct Nullable<'a, T, B: Buffer> {
//     _inner: T,
//     _bitmap_buf: <B as Buffer>::Container<'a, u8>,
// }
// trait Validity<const NULLABLE: bool = false> {
//     type Storage<'a, X: Buffer>;
// }
// impl<T> Validity<false> for T {
//     type Storage<'a, X: Buffer> = X;
// }
// impl<T> Validity<true> for T {
//     type Storage<'a, X: Buffer> = Nullable<'a, T, X>;
// }

// struct BooleanArray<'a, const NULLABLE: bool, D: Validity<NULLABLE> + Buffer, B: Buffer>(
//     <D as Validity<NULLABLE>>::Storage<'a, B>,
// );

// /// Trait to construct arrays.
// pub trait ArrayConstructor: ArrayType {
//     type Array<'a, const NULLABLE: bool, T: Buffer>;
// }

// /// Used to get the concrete array impl of something. Uses the ArrayConstructor trait.
// pub trait ArrayType {
//     type Array<'a, T: Buffer>;
// }

// impl<'a, const NULLABLE: bool, D: Validity<NULLABLE> + Buffer, B: Buffer> ArrayType
//     for BooleanArray<'a, NULLABLE, D, B>
// {
// }

// /// Implemented by arrays
// pub trait Array {}

// impl<'b, const X: bool, T: Buffer, U> ArrayConstructor for BooleanArray<'b, X, T, U>
// where
//     T: Validity<X>,
//     U: Buffer,
// {
//     type Array<'a, const NULLABLE: bool, T: Buffer> = BooleanArray<'a, NULLABLE, T, T>;
// }
