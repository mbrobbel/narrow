use std::fmt::Debug;

/// Subtrait for primitive types.
///
/// This exists to use as trait bound where one or more of the supertraits of
/// this trait are required, and to restrict certain implementations to Arrow
/// primitive types.
///
/// This trait is sealed to prevent downstream implementations.
pub trait Primitive: Copy + Debug + Default + sealed::Sealed {}

impl Primitive for i8 {}
impl Primitive for i16 {}
impl Primitive for i32 {}
impl Primitive for i64 {}
impl Primitive for u8 {}
impl Primitive for u16 {}
impl Primitive for u32 {}
impl Primitive for u64 {}
impl Primitive for f32 {}
impl Primitive for f64 {}

// Sealed traits.
mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for T where T: super::Primitive {}
}
