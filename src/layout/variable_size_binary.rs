use crate::{
    buffer::VecBuffer, layout::variable_size_list::VariableSizeList, nullability::NonNullable,
};

/// Variable size binary layout.
pub type VariableSizeBinary<Nulls = NonNullable, OffsetItem = i32, Storage = VecBuffer> =
    VariableSizeList<u8, Nulls, OffsetItem, Storage>;

#[cfg(test)]
mod tests {
    use crate::{collection::tests::round_trip, nullability::Nullable};

    use super::*;

    #[test]
    fn collection() {
        round_trip::<VariableSizeBinary, _>([vec![1, 2, 3, 4], vec![5, 6, 7, 8]]);
        round_trip::<VariableSizeBinary<Nullable>, _>([Some(vec![1, 2, 3, 4]), None]);
    }
}
