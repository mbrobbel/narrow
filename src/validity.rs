//! Validity trait for nullable and non-nullable data.

use crate::nullable::Nullable;

pub trait Validity<const NULLABLE: bool> {
    type Storage<BitmapBuffer>;
}

impl<T> Validity<false> for T {
    type Storage<BitmapBuffer> = T;
}

impl<T> Validity<true> for T {
    type Storage<BitmapBuffer> = Nullable<T, BitmapBuffer>;
}
