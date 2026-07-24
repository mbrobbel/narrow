use narrow::{array::Array, bitmap::ValidityBitmap, buffer::BufferRef, collection::ChildRef};

#[test]
fn borrows_nested_backing_memory() {
    type Nested = Option<Vec<Option<[i32; 2]>>>;

    let array = [Some(vec![Some([1, 2]), None])]
        .into_iter()
        .collect::<Array<Nested>>();

    let list = array.buffer_ref();
    let list_validity = list.buffer_ref();
    assert_eq!(
        list_validity
            .bitmap_ref()
            .expect("explicit validity")
            .buffer_ref()
            .as_slice(),
        &[0b0000_0001]
    );
    assert_eq!(
        list_validity
            .bitmap_ref()
            .expect("explicit validity")
            .bit_offset(),
        0
    );
    assert_eq!(list_validity.null_count(), 0);

    let offsets = list_validity.child_ref();
    assert_eq!(offsets.buffer_ref().as_slice(), &[0, 2]);

    let items = offsets.child_ref();
    let item_validity = items.buffer_ref();
    assert_eq!(
        item_validity
            .bitmap_ref()
            .expect("explicit validity")
            .buffer_ref()
            .as_slice(),
        &[0b0000_0001]
    );
    assert_eq!(item_validity.null_count(), 1);
    assert_eq!(item_validity.is_valid(0), Some(true));
    assert_eq!(item_validity.is_null(1), Some(true));

    let flattened = item_validity.child_ref();
    assert_eq!(flattened.child_ref().buffer_ref().as_slice(), &[1, 2, 0, 0]);
}

#[test]
fn borrows_boolean_backing_memory() {
    let array = [true, false, true].into_iter().collect::<Array<bool>>();
    let bitmap = array.buffer_ref().buffer_ref();

    assert_eq!(bitmap.buffer_ref().as_slice(), &[0b0000_0101]);
    assert_eq!(bitmap.bit_offset(), 0);
}
