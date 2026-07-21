use std::cell::Cell;

use narrow::{
    collection::{AllocError, Collection, CollectionAllocIn},
    length::Length,
};

#[derive(Debug)]
struct FixedArena<const N: usize> {
    values: [Cell<u32>; N],
    next: Cell<usize>,
}

impl<const N: usize> FixedArena<N> {
    fn new() -> Self {
        Self {
            values: core::array::from_fn(|_| Cell::new(0)),
            next: Cell::new(0),
        }
    }

    fn try_allocate(&self, capacity: usize) -> Result<usize, AllocError> {
        let start = self.next.get();
        let end = start.checked_add(capacity).ok_or(AllocError)?;
        if end > N {
            return Err(AllocError);
        }
        self.next.set(end);
        Ok(start)
    }

    fn remaining(&self) -> usize {
        N - self.next.get()
    }
}

#[derive(Debug)]
struct ArenaCollection<'arena, const N: usize> {
    arena: &'arena FixedArena<N>,
    start: usize,
    len: usize,
}

impl<const N: usize> Length for ArenaCollection<'_, N> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'arena, const N: usize> Collection for ArenaCollection<'arena, N> {
    type View<'collection>
        = u32
    where
        Self: 'collection;
    type Owned = u32;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        if index >= self.len {
            return None;
        }
        self.arena.values.get(self.start + index).map(Cell::get)
    }

    type Iter<'collection>
        = ArenaIter<'arena, N>
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        ArenaIter {
            arena: self.arena,
            next: self.start,
            end: self.start + self.len,
        }
    }

    type IntoIter = ArenaIter<'arena, N>;

    fn into_iter_owned(self) -> Self::IntoIter {
        self.iter_views()
    }
}

impl<'arena, const N: usize> CollectionAllocIn for ArenaCollection<'arena, N> {
    type Alloc = &'arena FixedArena<N>;

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
        let start = arena.try_allocate(capacity)?;
        Ok(Self {
            arena,
            start,
            len: 0,
        })
    }

    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        arena: Self::Alloc,
    ) -> Result<Self, AllocError> {
        let mut values = [0; N];
        let mut len = 0;
        for value in iter {
            if len == N {
                return Err(AllocError);
            }
            values[len] = value;
            len += 1;
        }

        let start = arena.try_allocate(len)?;
        for (index, value) in values.into_iter().take(len).enumerate() {
            arena.values[start + index].set(value);
        }
        Ok(Self { arena, start, len })
    }
}

#[derive(Debug)]
struct ArenaIter<'arena, const N: usize> {
    arena: &'arena FixedArena<N>,
    next: usize,
    end: usize,
}

impl<const N: usize> Iterator for ArenaIter<'_, N> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.end {
            return None;
        }
        let value = self.arena.values.get(self.next).map(Cell::get);
        self.next += 1;
        value
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<const N: usize> ExactSizeIterator for ArenaIter<'_, N> {
    fn len(&self) -> usize {
        self.end - self.next
    }
}

#[test]
fn collection_alloc_in_uses_a_fixed_preallocated_arena() {
    let arena = FixedArena::<6>::new();
    let first =
        ArenaCollection::try_from_iter_in([1, 2, 3], &arena).expect("first allocation fits");
    let second = ArenaCollection::try_from_iter_in([4, 5], &arena).expect("second allocation fits");

    assert_eq!(first.iter_views().collect::<Vec<_>>(), [1, 2, 3]);
    assert_eq!(second.into_iter_owned().collect::<Vec<_>>(), [4, 5]);
    assert_eq!(arena.remaining(), 1);

    assert!(matches!(
        ArenaCollection::<'_, 6>::try_with_capacity_in(2, &arena),
        Err(AllocError)
    ));
    assert_eq!(arena.remaining(), 1);
}
