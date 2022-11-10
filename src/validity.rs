//! Validity trait for nullable and non-nullable data.

use crate::{buffer::Buffer, nullable::Nullable};

pub trait Validity<const NULLABLE: bool> {
    type Storage<BitmapBuffer: Buffer<u8>>;
}

impl<T> Validity<false> for T {
    type Storage<BitmapBuffer: Buffer<u8>> = T;
}

impl<T> Validity<true> for T {
    type Storage<BitmapBuffer: Buffer<u8>> = Nullable<T, BitmapBuffer>;
}
