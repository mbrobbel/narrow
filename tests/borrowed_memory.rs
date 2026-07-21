use core::mem::{align_of, size_of};

use narrow::{array::Array, fixed_size::FixedSizeArray};

#[test]
fn borrows_nested_backing_memory() {
    type Nested = Option<Vec<Option<[i32; 2]>>>;

    let array = [Some(vec![Some([1, 2]), None])]
        .into_iter()
        .collect::<Array<Nested>>();

    let list = array.buffer();
    let list_validity = list.buffer();
    assert_eq!(list_validity.bitmap().buffer().as_slice(), &[0b0000_0001]);
    assert_eq!(list_validity.bitmap().bit_offset(), 0);

    let offsets = list_validity.collection();
    assert_eq!(offsets.offsets().as_slice(), &[0, 2]);

    let items = offsets.data();
    let item_validity = items.buffer();
    assert_eq!(item_validity.bitmap().buffer().as_slice(), &[0b0000_0001]);

    let flattened = item_validity.collection();
    assert_eq!(flattened.child().buffer().as_slice(), &[1, 2, 0, 0]);
}

#[test]
fn borrows_boolean_backing_memory() {
    let array = [true, false, true].into_iter().collect::<Array<bool>>();
    let bitmap = array.buffer().buffer();

    assert_eq!(bitmap.buffer().as_slice(), &[0b0000_0101]);
    assert_eq!(bitmap.bit_offset(), 0);
}

#[test]
fn fixed_size_array_is_transparent() {
    assert_eq!(size_of::<FixedSizeArray<u16, 4>>(), size_of::<[u16; 4]>());
    assert_eq!(align_of::<FixedSizeArray<u16, 4>>(), align_of::<[u16; 4]>());
}
