struct Foo;
/// Array with [Foo] values.
struct RawFooArray<
    const _NARROW_NULLABLE: bool = false,
    _NARROW_VALIDITY_BITMAP_BUFFER = Vec<u8>,
>(
    narrow::array::null::NullArray<Foo, _NARROW_NULLABLE, _NARROW_VALIDITY_BITMAP_BUFFER>,
);
