//! Arrays whose buffers allocate with a [`std::alloc::Allocator`].

use std::{
    alloc::{AllocError, Allocator, Global, Layout},
    cell::Cell,
    ptr::NonNull,
};

use narrow::{
    array::Array,
    buffer::{BoxBuffer, VecBuffer},
    collection::{Collection, CollectionAllocIn, CollectionRealloc, TryReserveError},
    length::Length,
};

/// Forwards to [`Global`] and counts the allocations that are still live.
#[derive(Clone, Copy, Debug)]
struct Counting<'count>(&'count Cell<isize>);

// SAFETY: Every operation forwards to `Global` with the same arguments.
unsafe impl Allocator for Counting<'_> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let memory = Global.allocate(layout)?;
        self.0.set(self.0.get().strict_add(1));
        Ok(memory)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.0.set(self.0.get().strict_sub(1));
        // SAFETY: `ptr` was allocated by `Global` with `layout`, as the caller
        // guarantees it was allocated by this allocator.
        unsafe { Global.deallocate(ptr, layout) };
    }
}

/// Forwards to [`Global`] until its budget of allocations runs out.
#[derive(Clone, Copy, Debug)]
struct Limited<'budget>(&'budget Cell<usize>);

// SAFETY: Every operation that succeeds forwards to `Global` with the same
// arguments.
unsafe impl Allocator for Limited<'_> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let remaining = self.0.get().checked_sub(1).ok_or(AllocError)?;
        self.0.set(remaining);
        Global.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // SAFETY: `ptr` was allocated by `Global` with `layout`, as the caller
        // guarantees it was allocated by this allocator.
        unsafe { Global.deallocate(ptr, layout) };
    }
}

/// Forwards to [`Global`] but fails to shrink, which aborts code that shrinks
/// infallibly.
#[derive(Clone, Copy, Debug)]
struct NoShrink;

// SAFETY: Every operation that succeeds forwards to `Global` with the same
// arguments.
unsafe impl Allocator for NoShrink {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Global.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // SAFETY: `ptr` was allocated by `Global` with `layout`, as the caller
        // guarantees it was allocated by this allocator.
        unsafe { Global.deallocate(ptr, layout) };
    }

    unsafe fn shrink(
        &self,
        _: NonNull<u8>,
        _: Layout,
        _: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }
}

/// An allocator that never provides memory.
#[derive(Clone, Copy, Debug)]
struct Failing;

// SAFETY: It never returns memory, so there is nothing to deallocate.
unsafe impl Allocator for Failing {
    fn allocate(&self, _: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }

    unsafe fn deallocate(&self, _: NonNull<u8>, _: Layout) {
        unreachable!("no memory was allocated");
    }
}

#[test]
fn nested_arrays_allocate_every_buffer_with_the_allocator() {
    type Lists<'count> = Array<Option<Vec<i32>>, VecBuffer<Counting<'count>>>;

    let live = Cell::new(0);
    {
        let mut lists = Lists::from_iter_in([Some(vec![1, 2]), None], Counting(&live));
        // Validity, offsets and values.
        assert_eq!(live.get(), 3);

        lists
            .try_extend([Some(vec![3])])
            .expect("allocation succeeds");
        assert_eq!(
            lists.into_iter_owned().collect::<Vec<_>>(),
            [Some(vec![1, 2]), None, Some(vec![3])]
        );
    }
    assert_eq!(live.get(), 0);
}

#[test]
fn boxed_arrays_allocate_with_the_allocator() {
    let live = Cell::new(0);
    {
        let array = Array::<i32, BoxBuffer<Counting<'_>>>::from_iter_in([1, 2, 3], Counting(&live));
        assert_eq!(live.get(), 1);
        assert_eq!(array.iter_views().collect::<Vec<_>>(), [1, 2, 3]);
    }
    assert_eq!(live.get(), 0);
}

#[test]
fn allocator_failures_are_reported() {
    assert_eq!(
        Array::<i32, VecBuffer<Failing>>::try_from_iter_in([1], Failing).map(|_| ()),
        Err(TryReserveError::AllocError(AllocError))
    );
    let empty = Array::<i32, VecBuffer<Failing>>::try_from_iter_in([], Failing)
        .expect("an empty array allocates nothing");
    assert_eq!(empty.len(), 0);
}

#[test]
fn failed_extends_keep_the_array_unchanged() {
    type Lists<'budget> = Array<Option<Vec<i32>>, VecBuffer<Limited<'budget>>>;

    // Enough for the validity, offsets and values of the initial array, and
    // for growing one of them before another fails to grow.
    let budget = Cell::new(4);
    let mut lists = Lists::try_from_iter_in([Some(vec![1, 2]), None], Limited(&budget))
        .expect("allocation succeeds");
    assert_eq!(budget.get(), 1);

    assert_eq!(
        lists.try_extend([Some(vec![3; 64]), None]),
        Err(TryReserveError::AllocError(AllocError))
    );
    assert_eq!(
        lists.into_iter_owned().collect::<Vec<_>>(),
        [Some(vec![1, 2]), None]
    );
}

#[test]
fn boxed_arrays_report_allocator_failures() {
    assert_eq!(
        Array::<i32, BoxBuffer<Failing>>::try_from_iter_in([1], Failing).map(|_| ()),
        Err(TryReserveError::AllocError(AllocError))
    );

    // An iterator whose size hint is too low leaves spare capacity to drop.
    let live = Cell::new(0);
    {
        let items = [1, 2, 3].into_iter().filter(|_| true);
        let array = Array::<i32, BoxBuffer<Counting<'_>>>::try_from_iter_in(items, Counting(&live))
            .expect("allocation succeeds");
        assert_eq!(live.get(), 1);
        assert_eq!(array.iter_views().collect::<Vec<_>>(), [1, 2, 3]);
    }
    assert_eq!(live.get(), 0);
}

#[test]
fn boxed_arrays_do_not_shrink_in_place() {
    // An iterator whose size hint is too low leaves spare capacity, which a
    // boxed slice must not shrink in place with an allocator that can fail.
    let items = [1, 2, 3].into_iter().filter(|_| true);
    let array = Array::<i32, BoxBuffer<NoShrink>>::try_from_iter_in(items, NoShrink)
        .expect("allocation succeeds");
    assert_eq!(array.iter_views().collect::<Vec<_>>(), [1, 2, 3]);
}
