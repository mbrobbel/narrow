use crate::{
    buffer::VecBuffer, layout::variable_size_list::VariableSizeList, nullability::NonNullable,
};

/// Variable-size binary layout.
///
/// # Design
///
/// Arrow binary data has the same physical shape as a variable-size list of
/// bytes. This alias makes that equivalence explicit and reuses the list's
/// offsets, storage, nullability, and collection behavior.
///
/// # Examples
///
/// ```
/// use narrow::{collection::Collection, layout::variable_size_binary::VariableSizeBinary};
///
/// let values = [b"hi".to_vec()].into_iter().collect::<VariableSizeBinary>();
/// assert_eq!(values.owned(0), Some(b"hi".to_vec()));
/// ```
pub type VariableSizeBinary<Nulls = NonNullable, OffsetItem = i32, Storage = VecBuffer> =
    VariableSizeList<u8, Nulls, OffsetItem, Storage>;

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::vec;

    use crate::{collection::tests::round_trip, nullability::Nullable};

    use super::*;

    #[test]
    fn collection() {
        round_trip::<VariableSizeBinary, _>([vec![1, 2, 3, 4], vec![5, 6, 7, 8]]);
        round_trip::<VariableSizeBinary<Nullable>, _>([Some(vec![1, 2, 3, 4]), None]);
    }
}
