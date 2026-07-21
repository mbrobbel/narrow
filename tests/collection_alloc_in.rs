use std::{
    borrow::Borrow,
    cell::{Cell, RefCell, RefMut},
    marker::PhantomData,
    slice,
};

use narrow::{
    array::Array,
    buffer::Buffer,
    collection::{AllocError, Collection, CollectionAllocIn},
    fixed_size::FixedSize,
    length::Length,
};

/// A fixed arena split into independently borrowed chunks.
#[derive(Debug)]
struct FixedArena<T: FixedSize, const CAPACITY: usize, const CHUNKS: usize> {
    chunks: [RefCell<[T; CAPACITY]>; CHUNKS],
    next: Cell<usize>,
}

impl<T: FixedSize, const CAPACITY: usize, const CHUNKS: usize> FixedArena<T, CAPACITY, CHUNKS> {
    /// Preallocates every chunk and initializes its otherwise unused values.
    fn new(initial: T) -> Self {
        Self {
            chunks: core::array::from_fn(|_| RefCell::new([initial; CAPACITY])),
            next: Cell::new(0),
        }
    }

    /// Claims the next chunk when it can satisfy the requested capacity.
    fn try_allocate(&self, capacity: usize) -> Result<RefMut<'_, [T; CAPACITY]>, AllocError> {
        if capacity > CAPACITY {
            return Err(AllocError);
        }

        let index = self.next.get();
        let chunk = self.chunks.get(index).ok_or(AllocError)?;
        let values = chunk.try_borrow_mut().map_err(|_| AllocError)?;
        self.next.set(index + 1);
        Ok(values)
    }

    /// Returns how many collections can still be allocated.
    fn remaining_chunks(&self) -> usize {
        CHUNKS - self.next.get()
    }
}

/// A collection backed by one exclusively borrowed arena chunk.
#[derive(Debug)]
struct ArenaCollection<'arena, T: FixedSize, const CAPACITY: usize, const CHUNKS: usize> {
    values: RefMut<'arena, [T; CAPACITY]>,
    len: usize,
    _chunks: PhantomData<[(); CHUNKS]>,
}

impl<T: FixedSize, const CAPACITY: usize, const CHUNKS: usize> Length
    for ArenaCollection<'_, T, CAPACITY, CHUNKS>
{
    fn len(&self) -> usize {
        self.len
    }
}

// `Buffer` requires its backing collection to expose contiguous initialized values.
impl<T: FixedSize, const CAPACITY: usize, const CHUNKS: usize> Borrow<[T]>
    for ArenaCollection<'_, T, CAPACITY, CHUNKS>
{
    fn borrow(&self) -> &[T] {
        &self.values[..self.len]
    }
}

// Only the initialized prefix of the claimed chunk is logically part of the collection.
impl<'arena, T: FixedSize, const CAPACITY: usize, const CHUNKS: usize> Collection
    for ArenaCollection<'arena, T, CAPACITY, CHUNKS>
{
    type View<'collection>
        = T
    where
        Self: 'collection;
    type Owned = T;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.values.get(..self.len)?.get(index).copied()
    }

    type Iter<'collection>
        = core::iter::Copied<slice::Iter<'collection, T>>
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        self.values[..self.len].iter().copied()
    }

    type IntoIter = ArenaIntoIter<'arena, T, CAPACITY>;

    fn into_iter_owned(self) -> Self::IntoIter {
        // Keep the chunk exclusively borrowed for the lifetime of the iterator.
        ArenaIntoIter {
            values: self.values,
            next: 0,
            end: self.len,
        }
    }
}

// The shared arena reference is the collection's cloneable allocator handle.
impl<'arena, T: FixedSize, const CAPACITY: usize, const CHUNKS: usize> CollectionAllocIn
    for ArenaCollection<'arena, T, CAPACITY, CHUNKS>
{
    type Alloc = &'arena FixedArena<T, CAPACITY, CHUNKS>;

    fn with_capacity_in(capacity: usize, arena: Self::Alloc) -> Self {
        match Self::try_with_capacity_in(capacity, arena) {
            Ok(collection) => collection,
            Err(AllocError) => panic!("fixed arena exhausted"),
        }
    }

    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, arena: Self::Alloc) -> Self {
        match Self::try_from_iter_in(iter, arena) {
            Ok(collection) => collection,
            Err(AllocError) => panic!("fixed arena exhausted"),
        }
    }

    fn try_with_capacity_in(capacity: usize, arena: Self::Alloc) -> Result<Self, AllocError> {
        Ok(Self {
            values: arena.try_allocate(capacity)?,
            len: 0,
            _chunks: PhantomData,
        })
    }

    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        arena: Self::Alloc,
    ) -> Result<Self, AllocError> {
        // Stage values first so oversized input leaves the arena unchanged.
        let mut values = [None; CAPACITY];
        let mut len = 0;
        for value in iter {
            if len == CAPACITY {
                return Err(AllocError);
            }
            values[len] = Some(value);
            len += 1;
        }

        let mut collection = Self::try_with_capacity_in(len, arena)?;
        for (index, value) in values.into_iter().take(len).flatten().enumerate() {
            collection.values[index] = value;
        }
        collection.len = len;
        Ok(collection)
    }
}

/// An owning iterator that retains the arena chunk guard until it is dropped.
#[derive(Debug)]
struct ArenaIntoIter<'arena, T: FixedSize, const CAPACITY: usize> {
    values: RefMut<'arena, [T; CAPACITY]>,
    next: usize,
    end: usize,
}

impl<T: FixedSize, const CAPACITY: usize> Iterator for ArenaIntoIter<'_, T, CAPACITY> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.end {
            return None;
        }
        let value = self.values.get(self.next).copied();
        self.next += 1;
        value
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<T: FixedSize, const CAPACITY: usize> ExactSizeIterator for ArenaIntoIter<'_, T, CAPACITY> {
    fn len(&self) -> usize {
        self.end - self.next
    }
}

/// Selects [`ArenaCollection`] as the fixed-size backing collection for an [`Array`].
#[derive(Clone, Copy, Debug, Default)]
struct ArenaBuffer<'arena, const CAPACITY: usize, const CHUNKS: usize>(PhantomData<&'arena ()>);

impl<'arena, const CAPACITY: usize, const CHUNKS: usize> Buffer
    for ArenaBuffer<'arena, CAPACITY, CHUNKS>
{
    type For<T: FixedSize> = ArenaCollection<'arena, T, CAPACITY, CHUNKS>;
}

#[test]
fn collection_alloc_in_uses_a_fixed_preallocated_arena() {
    let arena = FixedArena::<u32, 3, 2>::new(0);

    // Two live collections claim separate chunks from the same allocator.
    let first =
        ArenaCollection::try_from_iter_in([1, 2, 3], &arena).expect("first allocation fits");
    let second = ArenaCollection::try_from_iter_in([4, 5], &arena).expect("second allocation fits");

    assert_eq!(first.iter_views().collect::<Vec<_>>(), [1, 2, 3]);
    assert_eq!(second.into_iter_owned().collect::<Vec<_>>(), [4, 5]);
    assert_eq!(arena.remaining_chunks(), 0);

    // Exhaustion is recoverable and does not advance the arena cursor.
    assert!(matches!(
        ArenaCollection::try_with_capacity_in(1, &arena),
        Err(AllocError)
    ));
    assert_eq!(arena.remaining_chunks(), 0);
}

#[test]
fn array_uses_the_arena_collection_as_backing() {
    type Storage<'arena> = ArenaBuffer<'arena, 4, 1>;
    type ArenaArray<'arena> = Array<u32, Storage<'arena>>;

    let arena = FixedArena::<u32, 4, 1>::new(0);

    // Allocation flows through Array and its layout into the custom backing collection.
    let array = ArenaArray::try_from_iter_in([10, 20, 30], &arena).expect("array allocation fits");

    assert_eq!(array.iter_views().collect::<Vec<_>>(), [10, 20, 30]);
    assert_eq!(arena.remaining_chunks(), 0);
}
