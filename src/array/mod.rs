use crate::Data;

mod fixed_size_primitive;
pub use fixed_size_primitive::*;

mod boolean;
pub use boolean::*;

mod variable_size_binary;
pub use variable_size_binary::*;

/// Types storing sequences of values.
pub trait Array {
    /// Data type that stores the values of the array.
    type Data: Data;

    /// Returns a reference to the array's data.
    fn data(&self) -> &Self::Data;

    /// Returns the number of elements in the array.
    fn len(&self) -> usize {
        self.data().len()
    }

    /// Returns [true] when the array has a length of 0.
    fn is_empty(&self) -> bool {
        self.data().is_empty()
    }

    /// Returns the number of null values in this array.
    fn null_count(&self) -> usize {
        self.data().null_count()
    }

    /// Returns the number of non-null values in this array.
    fn valid_count(&self) -> usize {
        self.data().valid_count()
    }
}
