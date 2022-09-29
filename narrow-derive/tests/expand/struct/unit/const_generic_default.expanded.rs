pub struct Foo<const N: usize = 42>;
/// Array with [Foo] values.
pub struct RawFooArray<
    const N: usize = 42,
    const _NARROW_NULLABLE: bool = false,
    _NARROW_VALIDITY_BITMAP_BUFFER = Vec<u8>,
>(
    narrow::array::null::NullArray<
        Foo<N>,
        _NARROW_NULLABLE,
        _NARROW_VALIDITY_BITMAP_BUFFER,
    >,
);
