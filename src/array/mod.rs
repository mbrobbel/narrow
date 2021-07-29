mod fixed_size_primitive;
pub use fixed_size_primitive::*;

mod boolean;
pub use boolean::*;

mod variable_size_binary;
pub use variable_size_binary::*;

mod string;
pub use string::*;

mod variable_size_list;
pub use variable_size_list::*;

mod fixed_size_list;
pub use fixed_size_list::*;

mod r#struct;
pub use r#struct::*;

mod union;
pub use union::*;

mod null;
pub use null::*;

mod dictionary;
pub use dictionary::*;

/// Types for which sequences of values can be stored in arrays.
pub trait ArrayType {
    /// Array type used for this type.
    type Array: Array;
}

/// A sequence of values with known length all having the same type.
// todo(mb): https://github.com/rust-lang/rust/issues/20671
pub trait Array {
    /// [Validity](crate::Validity) of the array.
    // todo(mb): GATs
    type Validity: ArrayData;

    /// Returns a reference to the [Validity](crate::Validity) of the array.
    fn validity(&self) -> &Self::Validity;

    /// Returns the number of elements in the array, also referred to as its
    /// length.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Array, Uint32Array};
    /// let array: Uint32Array<false> = [1, 2, 3, 4].into_iter().collect();
    /// assert_eq!(array.len(), 4);
    /// ```
    fn len(&self) -> usize {
        self.validity().len()
    }

    /// Returns `true` if the array contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Array, Uint32Array};
    /// let empty: Uint32Array<false> = [].into_iter().collect();
    /// assert!(empty.is_empty());
    /// ```
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of null elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Array, Uint32Array};
    /// let array: Uint32Array<true> = [Some(1), None, Some(3), Some(4)].into_iter().collect();
    /// assert_eq!(array.null_count(), 1);
    /// ```
    fn null_count(&self) -> usize {
        self.validity().null_count()
    }

    /// Returns `true` if the array contains at least one null element.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Array, Uint32Array};
    /// let array: Uint32Array<true> = [Some(1), None, Some(3), Some(4)].into_iter().collect();
    /// assert!(array.any_null());
    /// ```
    fn any_null(&self) -> bool {
        self.null_count() > 0
    }

    /// Returns `true` if all the elements in the array are null.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Array, Uint32Array};
    /// let array: Uint32Array<true> = [None, None, None, None].into_iter().collect();
    /// assert!(array.all_null());
    /// ```
    fn all_null(&self) -> bool {
        self.null_count() == self.len()
    }

    /// Returns `true` if the element at position `index` is null.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Array, Uint32Array};
    /// let array: Uint32Array<true> = [Some(1), None, Some(3), Some(4)].into_iter().collect();
    /// assert!(!array.is_null(0));
    /// assert!(array.is_null(1));
    /// ```
    fn is_null(&self, index: usize) -> bool {
        self.validity().is_null(index)
    }

    /// Returns the number of valid elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Array, Uint32Array};
    /// let array: Uint32Array<true> = [Some(1), None, Some(3), Some(4)].into_iter().collect();
    /// assert_eq!(array.valid_count(), 3);
    /// ```
    fn valid_count(&self) -> usize {
        self.len() - self.null_count()
    }

    /// Returns `true` if the array contains at least one valid element.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Array, Uint32Array};
    /// let array: Uint32Array<true> = [Some(1), None, Some(3), Some(4)].into_iter().collect();
    /// assert!(array.any_valid());
    /// ```
    fn any_valid(&self) -> bool {
        self.valid_count() > 0
    }

    /// Returns `true` if all the elements in the array are valid.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Array, Uint32Array};
    /// let array: Uint32Array<true> = [Some(1), Some(2), Some(3), Some(4)].into_iter().collect();
    /// assert!(array.all_valid());
    /// ```
    fn all_valid(&self) -> bool {
        self.valid_count() == self.len()
    }

    /// Returns `true` if the element at position `index` is valid.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use narrow::{Array, Uint32Array};
    /// let array: Uint32Array<true> = [Some(1), None, Some(3), Some(4)].into_iter().collect();
    /// assert!(array.is_valid(0));
    /// assert!(!array.is_valid(1));
    /// ```
    fn is_valid(&self, index: usize) -> bool {
        !self.is_null(index)
    }
}

// Not part of Array trait because there are not GATs yet.
/// Index trait to get owned values of an array.
pub trait ArrayIndex<T> {
    type Output;
    fn index(&self, index: T) -> Self::Output;
}

/// Types storing nested sequences of values.
pub trait NestedArray {
    type Child: Array;

    /// Returns a reference to the child array.
    fn child(&self) -> &Self::Child;
}

/// Types storing array data.
pub trait ArrayData {
    /// Returns the number of elements.
    fn len(&self) -> usize;

    /// Returns `true` if the number of elements is `0`.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the element at position `index` is null.
    fn is_null(&self, index: usize) -> bool {
        !self.is_valid(index)
    }

    /// Returns the number of null elements.
    fn null_count(&self) -> usize {
        self.len() - self.valid_count()
    }

    // Returns `true` if the element at position `index` is valid.
    fn is_valid(&self, index: usize) -> bool;

    /// Returns the number of valid elements.
    fn valid_count(&self) -> usize;
}

impl<T> ArrayData for T
where
    T: Array,
{
    fn len(&self) -> usize {
        Array::len(self)
    }

    fn is_null(&self, index: usize) -> bool {
        Array::is_null(self, index)
    }

    fn null_count(&self) -> usize {
        Array::null_count(self)
    }

    fn is_valid(&self, index: usize) -> bool {
        Array::is_valid(self, index)
    }

    fn valid_count(&self) -> usize {
        Array::valid_count(self)
    }
}
