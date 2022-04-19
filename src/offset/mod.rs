//! Offsets for variable sized arrays.

use std::{num::TryFromIntError, ops::AddAssign};

use crate::Primitive;

/// Types representing offset values.
///
/// Values with these types can be used to represent offset values.
///
/// This trait is sealed to prevent downstream implementations.
pub trait OffsetValue:
    Primitive
    + AddAssign
    + TryFrom<usize, Error = TryFromIntError>
    + TryInto<usize, Error = TryFromIntError>
    + sealed::Sealed
{
}

mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for T where T: super::OffsetValue {}
}

impl OffsetValue for i32 {}
impl OffsetValue for i64 {}
