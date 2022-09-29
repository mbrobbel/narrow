pub(super) struct Foo<const N: bool = false>
where
    Self: Sized;
/// Array with [Foo] values.
pub(super) struct RawFooArray<
    const N: bool = false,
    const _NARROW_NULLABLE: bool = false,
    _NARROW_VALIDITY_BITMAP_BUFFER = Vec<u8>,
>(
    narrow::array::null::NullArray<
        Foo<N>,
        _NARROW_NULLABLE,
        _NARROW_VALIDITY_BITMAP_BUFFER,
    >,
)
where
    Self: Sized;
