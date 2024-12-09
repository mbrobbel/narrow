//! Interop with [`arrow-rs`] string array.

use std::sync::Arc;

use arrow_array::OffsetSizeTrait;
use arrow_buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow_schema::{DataType, Field};

use crate::{
    array::{Array, VariableSizeListArray},
    arrow::Offset,
    bitmap::Bitmap,
    buffer::BufferType,
    nullability::{NonNullable, Nullability, Nullable},
    offset::Offsets,
    validity::Validity,
};

impl<
        T: crate::arrow::Array,
        Nullable: Nullability,
        OffsetItem: Offset + OffsetSizeTrait,
        Buffer: BufferType,
    > crate::arrow::Array for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
{
    type Array = arrow_array::GenericListArray<OffsetItem>;

    fn as_field(name: &str) -> arrow_schema::Field {
        Field::new(name, Self::data_type(), Nullable::NULLABLE)
    }

    fn data_type() -> arrow_schema::DataType {
        if OffsetItem::LARGE {
            DataType::LargeList(Arc::new(T::as_field("item")))
        } else {
            DataType::List(Arc::new(T::as_field("item")))
        }
    }
}

impl<T: Array, Nullable: Nullability, OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<Arc<dyn arrow_array::Array>> for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
where
    Self: From<arrow_array::GenericListArray<OffsetItem>>,
{
    fn from(value: Arc<dyn arrow_array::Array>) -> Self {
        Self::from(arrow_array::GenericListArray::<OffsetItem>::from(
            value.to_data(),
        ))
    }
}

impl<T: Array + crate::arrow::Array, OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<VariableSizeListArray<T, NonNullable, OffsetItem, Buffer>> for Arc<dyn arrow_array::Array>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    <T as crate::arrow::Array>::Array: From<T> + 'static,
{
    fn from(value: VariableSizeListArray<T, NonNullable, OffsetItem, Buffer>) -> Self {
        let array: arrow_array::GenericListArray<OffsetItem> = value.into();
        Arc::new(array)
    }
}

impl<T: Array + crate::arrow::Array, OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<VariableSizeListArray<T, Nullable, OffsetItem, Buffer>> for Arc<dyn arrow_array::Array>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    Bitmap<Buffer>: Into<NullBuffer>,
    <T as crate::arrow::Array>::Array: From<T> + 'static,
{
    fn from(value: VariableSizeListArray<T, Nullable, OffsetItem, Buffer>) -> Self {
        let array: arrow_array::GenericListArray<OffsetItem> = value.into();
        Arc::new(array)
    }
}

impl<T: Array + crate::arrow::Array, OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<VariableSizeListArray<T, NonNullable, OffsetItem, Buffer>>
    for arrow_array::GenericListArray<OffsetItem>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    <T as crate::arrow::Array>::Array: From<T> + 'static,
{
    fn from(value: VariableSizeListArray<T, NonNullable, OffsetItem, Buffer>) -> Self {
        arrow_array::GenericListArray::new(
            Arc::new(T::as_field("item")),
            // Safety:
            // - The narrow offfset buffer contains valid offset data
            unsafe { OffsetBuffer::new_unchecked(value.0.offsets.into()) },
            Arc::<<T as crate::arrow::Array>::Array>::new(value.0.data.into()),
            None,
        )
    }
}

impl<T: Array + crate::arrow::Array, OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<VariableSizeListArray<T, Nullable, OffsetItem, Buffer>>
    for arrow_array::GenericListArray<OffsetItem>
where
    <Buffer as BufferType>::Buffer<OffsetItem>: Into<ScalarBuffer<OffsetItem>>,
    Bitmap<Buffer>: Into<NullBuffer>,
    <T as crate::arrow::Array>::Array: From<T> + 'static,
{
    fn from(value: VariableSizeListArray<T, Nullable, OffsetItem, Buffer>) -> Self {
        arrow_array::GenericListArray::new(
            Arc::new(T::as_field("item")),
            // Safety:
            // - The narrow offfset buffer contains valid offset data
            unsafe { OffsetBuffer::new_unchecked(value.0.offsets.data.into()) },
            Arc::<<T as crate::arrow::Array>::Array>::new(value.0.data.into()),
            Some(value.0.offsets.validity.into()),
        )
    }
}

/// Panics when there are nulls
impl<T: Array, OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<arrow_array::GenericListArray<OffsetItem>>
    for VariableSizeListArray<T, NonNullable, OffsetItem, Buffer>
where
    T: From<Arc<dyn arrow_array::Array>>,
    <Buffer as BufferType>::Buffer<OffsetItem>: From<ScalarBuffer<OffsetItem>>,
{
    fn from(value: arrow_array::GenericListArray<OffsetItem>) -> Self {
        let (_field, offsets, values, nulls_opt) = value.into_parts();
        match nulls_opt {
            Some(_) => panic!("expected array without a null buffer"),
            None => VariableSizeListArray(Offsets {
                data: values.into(),
                offsets: offsets.into_inner().into(),
            }),
        }
    }
}

impl<T: Array, OffsetItem: Offset + OffsetSizeTrait, Buffer: BufferType>
    From<arrow_array::GenericListArray<OffsetItem>>
    for VariableSizeListArray<T, Nullable, OffsetItem, Buffer>
where
    T: From<Arc<dyn arrow_array::Array>>,
    <Buffer as BufferType>::Buffer<OffsetItem>: From<ScalarBuffer<OffsetItem>>,
    Bitmap<Buffer>: From<NullBuffer> + FromIterator<bool>,
{
    fn from(value: arrow_array::GenericListArray<OffsetItem>) -> Self {
        let (_field, offsets_buffer, values, nulls_opt) = value.into_parts();
        let data = values.into();
        let offsets = offsets_buffer.into_inner().into();
        match nulls_opt {
            Some(null_buffer) => VariableSizeListArray(Offsets {
                data,
                offsets: Validity {
                    data: offsets,
                    validity: null_buffer.into(),
                },
            }),
            None => VariableSizeListArray::<T, NonNullable, OffsetItem, Buffer>(Offsets {
                data,
                offsets,
            })
            .into(),
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
        arrow::buffer::ScalarBuffer,
        bitmap::ValidityBitmap,
        Length, NonNullable, Nullable,
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
            .collect::<VariableSizeListArray<StringArray, Nullable>>();
        let list_array_nullable = arrow_array::ListArray::from(variable_size_list_array_nullable);
        assert_eq!(list_array_nullable.len(), INPUT_NULLABLE.len());
    }

    #[test]
    fn into_nullable() {
        let list_array = arrow_array::ListArray::from_iter_primitive::<UInt16Type, _, _>(
            INPUT
                .into_iter()
                .map(|opt| opt.iter().copied().map(Option::Some))
                .map(Option::Some),
        );
        assert!(!VariableSizeListArray::<
            Uint16Array<NonNullable, ScalarBuffer>,
            Nullable,
            i32,
            ScalarBuffer,
        >::from(list_array)
        .any_null());
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
            StringArray<NonNullable, i32, ScalarBuffer>,
            NonNullable,
            i32,
            ScalarBuffer,
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
            Uint16Array<NonNullable, ScalarBuffer>,
            NonNullable,
            i32,
            ScalarBuffer,
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
            StringArray<NonNullable, i32, ScalarBuffer>,
            Nullable,
            i32,
            ScalarBuffer,
        > = list_array_nullable.into();
    }
}
