//! Interop with [`arrow-rs`] string array.

use std::sync::Arc;

use arrow_array::OffsetSizeTrait;
use arrow_buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow_schema::{DataType, Field};

use crate::{
    array::{Array, VariableSizeListArray},
    arrow::ArrowArray,
    bitmap::Bitmap,
    buffer::BufferType,
    nullable::Nullable,
    offset::{Offset, OffsetElement},
    validity::Validity,
};

impl<
        T: ArrowArray,
        const NULLABLE: bool,
        OffsetItem: OffsetElement + OffsetSizeTrait,
        Buffer: BufferType,
    > ArrowArray for VariableSizeListArray<T, NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
{
    type Array = arrow_array::GenericListArray<OffsetItem>;

    fn as_field(name: &str) -> arrow_schema::Field {
        Field::new(
            name,
            DataType::List(Arc::new(T::as_field("item"))),
            NULLABLE,
        )
    }
}

impl<
        T: Array,
        const NULLABLE: bool,
        OffsetItem: OffsetElement + OffsetSizeTrait,
        Buffer: BufferType,
    > From<Arc<dyn arrow_array::Array>> for VariableSizeListArray<T, NULLABLE, OffsetItem, Buffer>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Validity<NULLABLE>,
    Self: From<arrow_array::GenericListArray<OffsetItem>>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::GenericListArray::<OffsetItem>::from(
            value.to_data(),
        ))
    }
}

impl<T: Array + ArrowArray, OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<VariableSizeListArray<T, false, OffsetItem, Buffer>>
    for arrow_array::GenericListArray<OffsetItem>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    <T as ArrowArray>::Array: From<T> + 'static,
{
    fn from(value: VariableSizeListArray<T, false, OffsetItem, Buffer>) -> Self {
        arrow_array::GenericListArray::new(
            Arc::new(T::as_field("item")),
            // Safety:
            // - The narrow offfset buffer contains valid offset data
            unsafe { OffsetBuffer::new_unchecked(value.0.offsets.into()) },
            Arc::<<T as ArrowArray>::Array>::new(value.0.data.into()),
            None,
        )
    }
}

impl<T: Array + ArrowArray, OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<VariableSizeListArray<T, true, OffsetItem, Buffer>>
    for arrow_array::GenericListArray<OffsetItem>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    Bitmap<Buffer>: Into<NullBuffer>,
    <T as ArrowArray>::Array: From<T> + 'static,
{
    fn from(value: VariableSizeListArray<T, true, OffsetItem, Buffer>) -> Self {
        arrow_array::GenericListArray::new(
            Arc::new(T::as_field("item")),
            // Safety:
            // - The narrow offfset buffer contains valid offset data
            unsafe { OffsetBuffer::new_unchecked(value.0.offsets.data.into()) },
            Arc::<<T as ArrowArray>::Array>::new(value.0.data.into()),
            Some(value.0.offsets.validity.into()),
        )
    }
}

/// Panics when there are nulls
impl<T: Array, OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<arrow_array::GenericListArray<OffsetItem>>
    for VariableSizeListArray<T, false, OffsetItem, Buffer>
where
    T: From<Arc<dyn arrow_array::Array>>,
    <Buffer as BufferType>::Buffer<OffsetItem>: From<ScalarBuffer<OffsetItem>>,
{
    fn from(value: arrow_array::GenericListArray<OffsetItem>) -> Self {
        let (_field, offsets, values, nulls_opt) = value.into_parts();
        match nulls_opt {
            Some(_) => panic!("expected array without a null buffer"),
            None => VariableSizeListArray(Offset {
                data: values.into(),
                offsets: offsets.into_inner().into(),
            }),
        }
    }
}

/// Panics when there are no nulls
impl<T: Array, OffsetItem: OffsetElement + OffsetSizeTrait, Buffer: BufferType>
    From<arrow_array::GenericListArray<OffsetItem>>
    for VariableSizeListArray<T, true, OffsetItem, Buffer>
where
    T: From<Arc<dyn arrow_array::Array>>,
    <Buffer as BufferType>::Buffer<OffsetItem>: From<ScalarBuffer<OffsetItem>>,
    Bitmap<Buffer>: From<NullBuffer>,
{
    fn from(value: arrow_array::GenericListArray<OffsetItem>) -> Self {
        let (_field, offsets, values, nulls_opt) = value.into_parts();
        match nulls_opt {
            Some(null_buffer) => VariableSizeListArray(Offset {
                data: values.into(),
                offsets: Nullable {
                    data: offsets.into_inner().into(),
                    validity: null_buffer.into(),
                },
            }),
            None => panic!("expected array with a null buffer"),
        }
    }
}

#[cfg(test)]
mod tests {
    use arrow_array::{
        builder::{ListBuilder, StringBuilder},
        types::UInt16Type,
        Array as _,
    };

    use crate::{
        array::{StringArray, Uint16Array, VariableSizeListArray},
        arrow::scalar_buffer::ArrowScalarBuffer,
        Length,
    };

    const INPUT: [&[u16]; 3] = [&[1, 2], &[3], &[4]];
    const INPUT_NULLABLE: [Option<&[&str]>; 3] =
        [Some(&["hello", " "]), None, Some(&["world", "!"])];

    #[test]
    fn from() {
        let variable_size_list_array = INPUT
            .into_iter()
            .collect::<VariableSizeListArray<Uint16Array>>();
        let list_array = arrow_array::ListArray::from(variable_size_list_array);
        assert_eq!(list_array.len(), INPUT.len());

        let variable_size_list_array_nullable = INPUT_NULLABLE
            .into_iter()
            .collect::<VariableSizeListArray<StringArray, true>>();
        let list_array_nullable = arrow_array::ListArray::from(variable_size_list_array_nullable);
        assert_eq!(list_array_nullable.len(), INPUT_NULLABLE.len());
    }

    #[test]
    #[should_panic(expected = "expected array with a null buffer")]
    fn into_nullable() {
        let list_array = arrow_array::ListArray::from_iter_primitive::<UInt16Type, _, _>(
            INPUT
                .into_iter()
                .map(|opt| opt.iter().copied().map(Option::Some))
                .map(Option::Some),
        );
        let _: VariableSizeListArray<
            Uint16Array<false, ArrowScalarBuffer>,
            true,
            i32,
            ArrowScalarBuffer,
        > = list_array.into();
    }

    #[test]
    #[should_panic(expected = "expected array without a null buffer")]
    fn into_non_nullable() {
        let mut list_builder =
            ListBuilder::with_capacity(StringBuilder::new(), INPUT_NULLABLE.len());
        INPUT_NULLABLE.into_iter().for_each(|opt| match opt {
            Some(items) => {
                for item in items {
                    list_builder.values().append_value(item);
                }
                list_builder.append(true);
            }
            None => {
                list_builder.append(false);
            }
        });
        let list_array_nullable = list_builder.finish();
        let _: VariableSizeListArray<
            StringArray<false, i32, ArrowScalarBuffer>,
            false,
            i32,
            ArrowScalarBuffer,
        > = list_array_nullable.into();
    }

    #[test]
    fn into() {
        let list_array = arrow_array::ListArray::from_iter_primitive::<UInt16Type, _, _>(
            INPUT
                .into_iter()
                .map(|opt| opt.iter().copied().map(Option::Some))
                .map(Option::Some),
        );
        let _: VariableSizeListArray<
            Uint16Array<false, ArrowScalarBuffer>,
            false,
            i32,
            ArrowScalarBuffer,
        > = list_array.into();

        let mut list_builder =
            ListBuilder::with_capacity(StringBuilder::new(), INPUT_NULLABLE.len());
        INPUT_NULLABLE.into_iter().for_each(|opt| match opt {
            Some(items) => {
                for item in items {
                    list_builder.values().append_value(item);
                }
                list_builder.append(true);
            }
            None => {
                list_builder.append(false);
            }
        });
        let list_array_nullable = list_builder.finish();
        let _: VariableSizeListArray<
            StringArray<false, i32, ArrowScalarBuffer>,
            true,
            i32,
            ArrowScalarBuffer,
        > = list_array_nullable.into();
    }
}
