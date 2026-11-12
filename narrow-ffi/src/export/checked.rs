//! Buffer pointers checked against the array that exports them.
//!
//! # Soundness
//!
//! Export accepts any safe [`Buffer`], whose collections may report a length
//! that differs from the slice they borrow as, or borrow as a different slice
//! on every call. Export stays sound because:
//!
//! - The exported layout is moved into an [`Arc`] root before any buffer is
//!   borrowed. Every exported array retains the root, and export never
//!   accesses it through `&mut`, so a slice borrowed from the root stays
//!   valid while the root lives.
//! - [`FixedSize`] is sealed to types without interior mutability, so such a
//!   slice also stays unchanged.
//! - Each buffer is borrowed once, and the exported pointer is that of the
//!   slice that was checked.
//! - Only this module creates [`BufferPointer`]s, [`Checker`]s and
//!   [`Exported`] arrays, and it checks every slice against the exported
//!   length and the lengths of the exported children.
//!
//! The export module trusts its layouts to pass only layouts that are part of
//! the root to [`array`], and to export each buffer of the Arrow layout.

extern crate alloc;

use alloc::{boxed::Box, sync::Arc};
use core::{borrow::Borrow, ffi::c_void, ptr};

use narrow::{
    bitmap::Bitmap,
    buffer::{Buffer, BufferRef},
    fixed_size::FixedSize,
};

use crate::{ArrowArray, ArrowListOffset};

use super::{ArrayData, ArrowArrayLayout, BufferKind, ExportError};

/// Pointer to a buffer whose slice holds the items the exported array
/// requires.
#[repr(transparent)]
pub(super) struct BufferPointer(*const c_void);

/// An [`ArrowArray`] whose buffers were checked by [`array`].
#[repr(transparent)]
pub(super) struct Exported(ArrowArray);

impl Exported {
    /// Returns the exported array.
    pub(super) fn into_inner(self) -> ArrowArray {
        self.0
    }

    /// Returns a pointer to the exported array.
    pub(super) fn as_mut_ptr(&mut self) -> *mut ArrowArray {
        &raw mut self.0
    }
}

/// Checks borrowed buffers against the length and children of an exported
/// array, and records the null count of its validity bitmap.
pub(super) struct Checker<'children> {
    /// Length of the exported array.
    length: usize,
    /// Exported children of the array.
    children: &'children [Exported],
    /// Number of unset bits in the exported validity bitmap.
    null_count: usize,
}

impl Checker<'_> {
    /// Returns the pointer of a validity bitmap that is absent because all
    /// items are valid.
    pub(super) fn all_valid(&mut self) -> BufferPointer {
        self.null_count = 0;
        BufferPointer(ptr::null())
    }

    /// Borrows an optional validity bitmap once and returns its pointer if it
    /// holds a bit for every item.
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::NonZeroOffset`] or
    /// [`ExportError::BufferTooShort`] when the bitmap cannot be exported.
    pub(super) fn validity<Storage: Buffer>(
        &mut self,
        validity: Option<&Bitmap<Storage>>,
    ) -> Result<BufferPointer, ExportError> {
        let Some(bitmap) = validity else {
            return Ok(self.all_valid());
        };
        let bytes = self.bitmap_bytes(bitmap, BufferKind::Validity)?;
        self.null_count = unset_bits(bytes, self.length);
        Ok(BufferPointer(bytes.as_ptr().cast()))
    }

    /// Borrows a bitmap of boolean values once and returns its pointer if it
    /// holds a bit for every item.
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::NonZeroOffset`] or
    /// [`ExportError::BufferTooShort`] when the bitmap cannot be exported.
    pub(super) fn bits<Storage: Buffer>(
        &self,
        bitmap: &Bitmap<Storage>,
    ) -> Result<BufferPointer, ExportError> {
        let bytes = self.bitmap_bytes(bitmap, BufferKind::Values)?;
        Ok(BufferPointer(bytes.as_ptr().cast()))
    }

    /// Borrows `buffer` once and returns its pointer if it holds a value for
    /// every item.
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::BufferTooShort`] when the buffer holds fewer
    /// values than the array has items.
    pub(super) fn values<T: FixedSize>(
        &self,
        buffer: &impl Borrow<[T]>,
    ) -> Result<BufferPointer, ExportError> {
        let values: &[T] = buffer.borrow();
        at_least(BufferKind::Values, self.length, values.len())?;
        Ok(BufferPointer(values.as_ptr().cast()))
    }

    /// Borrows the offsets of a list array once and returns their pointer if
    /// the first `length + 1` offsets start at zero or more, do not decrease
    /// and end within the only child.
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::BufferTooShort`] when there are too few offsets,
    /// [`ExportError::InvalidOffsets`] when they are negative or decrease, and
    /// [`ExportError::ChildTooShort`] when the last offset exceeds the child.
    pub(super) fn offsets<OffsetItem: ArrowListOffset>(
        &self,
        buffer: &impl Borrow<[OffsetItem]>,
    ) -> Result<BufferPointer, ExportError> {
        let all: &[OffsetItem] = buffer.borrow();
        let required = self.length.saturating_add(1);
        at_least(BufferKind::Offsets, required, all.len())?;
        let offsets = &all[..required];
        let first: i64 = offsets[0].into();
        if first < 0 {
            return Err(ExportError::InvalidOffsets { index: 0 });
        }
        if !offsets.is_sorted() {
            let index = offsets
                .windows(2)
                .position(|pair| pair[0] > pair[1])
                .map_or(0, |position| position.strict_add(1));
            return Err(ExportError::InvalidOffsets { index });
        }
        let last: i64 = offsets[self.length].into();
        let child = self.only_child().length;
        if last > child {
            return Err(ExportError::ChildTooShort {
                required: usize::try_from(last).unwrap_or(usize::MAX),
                actual: usize::try_from(child).expect("exported lengths are non-negative"),
            });
        }
        Ok(BufferPointer(offsets.as_ptr().cast()))
    }

    /// Checks that the only child holds `N` items for every item.
    ///
    /// # Errors
    ///
    /// Returns [`ExportError::ChildTooShort`] when the child is shorter.
    pub(super) fn fixed_size_child<const N: usize>(&self) -> Result<(), ExportError> {
        let required = N.saturating_mul(self.length);
        let actual =
            usize::try_from(self.only_child().length).expect("exported lengths are non-negative");
        if actual >= required {
            Ok(())
        } else {
            Err(ExportError::ChildTooShort { required, actual })
        }
    }

    /// Borrows the bytes of `bitmap` once and returns them if the bitmap has
    /// no bit offset and holds a bit for every item.
    fn bitmap_bytes<'bitmap, Storage: Buffer>(
        &self,
        bitmap: &'bitmap Bitmap<Storage>,
        buffer: BufferKind,
    ) -> Result<&'bitmap [u8], ExportError> {
        let offset = bitmap.bit_offset();
        if offset != 0 {
            return Err(ExportError::NonZeroOffset { offset });
        }
        let bytes: &[u8] = bitmap.buffer_ref().borrow();
        at_least(buffer, self.length, bytes.len().saturating_mul(8))?;
        Ok(bytes)
    }

    /// Returns the only exported child.
    fn only_child(&self) -> &ArrowArray {
        let [child] = <&[Exported; 1]>::try_from(self.children)
            .expect("list layouts export exactly one child")
            .each_ref();
        &child.0
    }
}

/// Checks that `buffer` holds at least `required` items.
fn at_least(buffer: BufferKind, required: usize, actual: usize) -> Result<(), ExportError> {
    if actual >= required {
        Ok(())
    } else {
        Err(ExportError::BufferTooShort {
            buffer,
            required,
            actual,
        })
    }
}

/// Counts the unset bits among the first `length` bits of `bytes`, which must
/// hold at least `length` bits.
fn unset_bits(bytes: &[u8], length: usize) -> usize {
    let (full, partial) = bytes.split_at(length / 8);
    let tail_bits = u32::try_from(length % 8).expect("bit index fits u32");
    let tail = partial
        .first()
        .map_or(0, |byte| byte & !u8::MAX.unbounded_shl(tail_bits));
    let (words, rest) = full.as_chunks::<8>();
    let set = words
        .iter()
        .map(|&word| u64::from_le_bytes(word).count_ones())
        .chain(rest.iter().chain([&tail]).map(|byte| byte.count_ones()))
        .map(u64::from)
        .sum::<u64>();
    length.strict_sub(usize::try_from(set).expect("bit count fits usize"))
}

/// Builds an [`ArrowArray`] for `layout`, which is part of `root`, from
/// buffers checked against its length and exported children.
pub(super) fn array<Layout, Root>(
    layout: &Layout,
    root: &Arc<Root>,
) -> Result<Exported, ExportError>
where
    Layout: ArrowArrayLayout + 'static,
    Root: Send + Sync + 'static,
{
    let length = layout.len();
    let children = layout.children(root)?;
    let mut checker = Checker {
        length,
        children: children.as_ref(),
        null_count: 0,
    };
    let buffers = layout.buffers(&mut checker)?;
    let null_count = checker.null_count;
    let private = Box::new(ArrayData::<Arc<Root>, Layout>::new(
        Arc::clone(root),
        buffers,
        children,
    ));

    Ok(Exported(private.into_array(
        i64::try_from(length).expect("array length exceeds i64"),
        i64::try_from(null_count).expect("null count exceeds i64"),
        layout.dictionary(),
    )))
}

#[cfg(test)]
mod tests {
    use super::unset_bits;

    #[test]
    fn counts_unset_bits_within_length() {
        let bytes = [0xff; 9];
        assert_eq!(unset_bits(&bytes, 0), 0);
        assert_eq!(unset_bits(&bytes, 72), 0);
        assert_eq!(unset_bits(&[0; 9], 72), 72);
        assert_eq!(unset_bits(&[0; 9], 69), 69);
        let mut mixed = [0b1010_1010; 10];
        mixed[9] = 0b1111_0101;
        assert_eq!(unset_bits(&mixed, 75), 36 + 1);
        assert_eq!(unset_bits(&mixed, 73), 36);
    }
}
