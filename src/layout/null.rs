//! The Arrow null layout: a length with no data or validity buffers.

use core::{
    iter::{RepeatN, repeat_n},
    marker::PhantomData,
};

use crate::{
    buffer::Buffer,
    collection::{
        AllocError, Collection, CollectionAllocIn, CollectionRealloc, owned::IntoOwned,
        view::AsView,
    },
    layout::{ArrayItem, MemoryLayout},
    length::Length,
};

/// A single-value type that can be reconstructed without storing any data.
///
/// Implementations must represent one value, reproduced by `Default`. This
/// includes `()` and fieldless structs. Implementing this trait alone does
/// not select an array layout; an [`ArrayItem`] implementation can select
/// `Nulls<Self>` explicitly. A [`crate::layout::Layout`] implementation can
/// instead select [`crate::layout::r#struct::Struct`] over `Nulls<Self>` to
/// represent an empty struct and enable nullable struct rows.
/// `IntoOwned<Self>` lets collection views return
/// the unit value directly. Nesting a custom unit in a variable-size list
/// also requires an [`AsView`] implementation for the item type.
///
/// This is a semantic contract, not a memory-safety requirement: no unsafe
/// code relies on it. Implementing it for a type with multiple values loses
/// information: inputs are discarded and reads reconstruct `T::default()`.
/// A type such as `struct Flag(bool)` must therefore not implement `Unit`.
///
/// ```
/// use narrow::{
///     array::Array, buffer::Buffer, collection::{Collection, owned::IntoOwned},
///     layout::{ArrayItem, null::{Nulls, Unit}},
/// };
///
/// #[derive(Clone, Copy, Default, Debug, PartialEq)]
/// struct Marker;
/// impl IntoOwned<Self> for Marker {
///     fn into_owned(self) -> Self { self }
/// }
/// impl Unit for Marker {}
/// impl ArrayItem for Marker {
///     type Memory<Storage: Buffer> = Nulls<Self>;
/// }
/// let values = [Marker; 2].into_iter().collect::<Array<Marker>>();
/// assert_eq!(values.owned(1), Some(Marker));
/// ```
pub trait Unit: Copy + Default + IntoOwned<Self> + 'static {}

impl Unit for () {}

impl ArrayItem for () {
    type Memory<Storage: Buffer> = Nulls;
}

impl IntoOwned<()> for () {
    fn into_owned(self) {}
}

impl<'a> AsView<'a> for () {
    type View = Self;
    fn as_view(&'a self) {}
}

/// A sequence containing only nulls, without allocated buffers.
///
/// Every slot is null, so the Arrow null count equals the length. Unlike
/// other layouts, this has no backing buffer and does not implement
/// `BufferRef`. Its allocator is `()`, independent of the selected storage;
/// nesting under a nullable or variable-size parent requires storage whose
/// allocator is also `()`.
///
/// ```
/// use narrow::{array::Array, collection::Collection, layout::null::Nulls, length::Length};
/// let values = [(); 3].into_iter().collect::<Array<()>>();
/// assert_eq!(values.len(), 3);
/// assert_eq!(values.owned(0), Some(()));
/// assert_eq!(values.owned(3), None);
/// let values = Array::<()>::from_buffer(Nulls::new(10));
/// assert_eq!(values.len(), 10);
/// ```
///
/// `Array<Option<()>>` is unsupported because the null layout cannot preserve the
/// distinction between `Some(())` and `None`.
///
/// ```compile_fail
/// use narrow::array::Array;
/// let _: Array<Option<()>>;
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nulls<T: Unit = ()> {
    len: usize,
    item: PhantomData<T>,
}

impl<T: Unit> Nulls<T> {
    /// Constructs a sequence of `len` nulls without allocating memory.
    #[must_use]
    pub const fn new(len: usize) -> Self {
        Self {
            len,
            item: PhantomData,
        }
    }
}

impl<T: Unit> Length for Nulls<T> {
    fn len(&self) -> usize {
        self.len
    }
}
impl<T: Unit> MemoryLayout for Nulls<T> {}
impl<T: Unit> Collection for Nulls<T> {
    type Owned = T;
    type View<'a> = T;
    type Iter<'a> = RepeatN<T>;
    type IntoIter = RepeatN<T>;
    fn view(&self, index: usize) -> Option<T> {
        (index < self.len).then(T::default)
    }
    fn iter_views(&self) -> Self::Iter<'_> {
        repeat_n(T::default(), self.len)
    }
    fn into_iter_owned(self) -> Self::IntoIter {
        repeat_n(T::default(), self.len)
    }
}
impl<T: Unit> Extend<T> for Nulls<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.try_extend(iter).expect("null length exceeds usize");
    }
}
impl<T: Unit> FromIterator<T> for Nulls<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut values = Self::default();
        values.extend(iter);
        values
    }
}
impl<T: Unit> CollectionAllocIn for Nulls<T> {
    type Alloc = ();
    fn with_capacity_in(_: usize, (): ()) -> Self {
        Self::default()
    }
    fn try_with_capacity_in(_: usize, (): ()) -> Result<Self, AllocError> {
        Ok(Self::default())
    }
    fn from_iter_in<I: IntoIterator<Item = T>>(iter: I, (): ()) -> Self {
        iter.into_iter().collect()
    }
    fn try_from_iter_in<I: IntoIterator<Item = T>>(iter: I, (): ()) -> Result<Self, AllocError> {
        let mut values = Self::default();
        values.try_extend(iter)?;
        Ok(values)
    }
}
impl<T: Unit> CollectionRealloc for Nulls<T> {
    fn allocator(&self) {}
    fn try_reserve(&mut self, additional: usize) -> Result<(), AllocError> {
        self.len
            .checked_add(additional)
            .map(|_| ())
            .ok_or(AllocError)
    }
    fn reserve(&mut self, additional: usize) {
        self.try_reserve(additional)
            .expect("null length exceeds usize");
    }
    fn try_extend<I: IntoIterator<Item = T>>(&mut self, iter: I) -> Result<(), AllocError> {
        let mut len = self.len;
        for _ in iter {
            len = len.checked_add(1).ok_or(AllocError)?;
        }
        self.len = len;
        Ok(())
    }
    fn truncate(&mut self, len: usize) {
        self.len = self.len.min(len);
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use crate::{array::Array, collection::tests::round_trip};
    use alloc::{vec, vec::Vec};
    #[test]
    fn collection() {
        round_trip::<Nulls, _>([(); 3]);
        round_trip::<Array<()>, _>([(); 3]);
        round_trip::<Array<()>, _>([]);
        round_trip::<Array<[(); 2]>, _>([[(); 2]; 3]);
        round_trip::<Array<Option<[(); 2]>>, _>([Some([(); 2]), None]);
        round_trip::<Array<Vec<()>>, _>([vec![], vec![(); 3], vec![()]]);
        round_trip::<Array<Option<Vec<()>>>, _>([Some(vec![()]), None]);
    }
    #[test]
    fn custom_unit() {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        struct Marker;
        impl IntoOwned<Self> for Marker {
            fn into_owned(self) -> Self {
                self
            }
        }
        impl Unit for Marker {}
        impl<'a> AsView<'a> for Marker {
            type View = Self;
            fn as_view(&'a self) -> Self {
                *self
            }
        }
        impl ArrayItem for Marker {
            type Memory<Storage: Buffer> = Nulls<Self>;
        }

        round_trip::<Array<Marker>, _>([Marker; 4]);
        round_trip::<Array<Vec<Marker>>, _>([vec![], vec![Marker; 2]]);
        assert_eq!(
            core::mem::size_of::<Nulls<Marker>>(),
            core::mem::size_of::<usize>()
        );
        let mut values =
            Nulls::<Marker>::try_with_capacity_in(usize::MAX, ()).expect("no allocation");
        assert!(values.is_empty());
        values.try_extend([Marker; 2]).expect("length fits");
        assert_eq!(values.owned(1), Some(Marker));
        assert_eq!(values.owned(2), None);
        let built = Nulls::<Marker>::try_from_iter_in([Marker; 2], ()).expect("length fits");
        assert_eq!(values, built);
    }

    #[test]
    fn bounds_and_growth() {
        let mut values = Nulls::<()>::new(usize::MAX);
        assert_eq!(values.view(usize::MAX), None);
        assert_eq!(values.view(usize::MAX - 1), Some(()));
        assert_eq!(values.try_reserve(1), Err(AllocError));
        assert_eq!(values.try_extend([()]), Err(AllocError));
        assert_eq!(values.len(), usize::MAX);
        let mut almost_full = Nulls::<()>::new(usize::MAX - 1);
        assert_eq!(almost_full.try_extend([(); 2]), Err(AllocError));
        assert_eq!(almost_full.len(), usize::MAX - 1);
        values.truncate(2);
        values.extend([()]);
        values.truncate(10);
        assert_eq!(values.len(), 3);
        let mut iter = values.into_iter_owned();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next_back(), Some(()));
        assert_eq!(iter.len(), 2);
    }
}
