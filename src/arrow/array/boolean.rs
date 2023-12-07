//! Interop with `arrow-rs` boolean array.

use crate::{array::BooleanArray, bitmap::Bitmap, buffer::BufferType};
use arrow_buffer::{BooleanBuffer, NullBuffer};

impl<Buffer: BufferType> From<BooleanArray<false, Buffer>> for arrow_array::BooleanArray
where
    Bitmap<Buffer>: Into<BooleanBuffer>,
{
    fn from(value: BooleanArray<false, Buffer>) -> Self {
        arrow_array::BooleanArray::new(value.0.into(), None)
    }
}

impl<Buffer: BufferType> From<BooleanArray<true, Buffer>> for arrow_array::BooleanArray
where
    Bitmap<Buffer>: Into<BooleanBuffer> + Into<NullBuffer>,
{
    fn from(value: BooleanArray<true, Buffer>) -> Self {
        arrow_array::BooleanArray::new(value.0.data.into(), Some(value.0.validity.into()))
    }
}

#[cfg(test)]
mod tests {
    use arrow_array::Array;

    use super::*;
    use crate::{bitmap::ValidityBitmap, Length};

    #[test]
    fn convert() {
        let input = [true, false, true, true];
        let array = input.into_iter().collect::<BooleanArray>();
        assert_eq!(array.len(), 4);
        let array_arrow: arrow_array::BooleanArray = array.into();
        assert_eq!(array_arrow.len(), 4);

        let input_nullable = [Some(true), None, Some(false)];
        let array_nullable = input_nullable.into_iter().collect::<BooleanArray<true>>();
        assert_eq!(array_nullable.len(), 3);
        assert_eq!(array_nullable.null_count(), 1);
        let array_arrow_nullable: arrow_array::BooleanArray = array_nullable.into();
        assert_eq!(array_arrow_nullable.len(), 3);
        assert_eq!(array_arrow_nullable.null_count(), 1);
    }
}
