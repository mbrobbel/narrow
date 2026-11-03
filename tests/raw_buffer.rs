//! A storage that is only `RawBuffer` (not host-readable) can still back Bitmap and Validity.
use narrow::{
    bitmap::Bitmap,
    buffer::{MemorySpace, RawBuffer, RawSlice, VecBuffer},
    fixed_size::FixedSize,
    length::Length,
    validity::Validity,
};
use std::marker::PhantomData;

struct Opaque<T> {
    addr: usize,
    len: usize,
    _t: PhantomData<T>,
}
impl<T> Length for Opaque<T> {
    fn len(&self) -> usize {
        self.len
    }
}
impl<T> RawSlice<T> for Opaque<T> {
    fn memory_space(&self) -> MemorySpace {
        MemorySpace::cuda(0)
    }
    fn address(&self) -> usize {
        self.addr
    }
}
struct OpaqueBuffer;
impl RawBuffer for OpaqueBuffer {
    type For<T: FixedSize> = Opaque<T>;
}

#[test]
fn raw_validity() {
    let bits = Bitmap::<OpaqueBuffer>::from_parts(
        Opaque {
            addr: 0x1000,
            len: 2,
            _t: PhantomData,
        },
        10,
        3,
    );
    let col = Validity::from_parts(
        Opaque::<i64> {
            addr: 0x2000,
            len: 10,
            _t: PhantomData,
        },
        bits,
    );
    assert_eq!(col.len(), 10);
    let (values, bitmap) = col.as_parts();
    assert_eq!(values.address(), 0x2000);
    let (bytes, n, off) = bitmap.unwrap().as_parts();
    assert_eq!(
        (bytes.address(), bytes.memory_space(), n, off),
        (0x1000, MemorySpace::cuda(0), 10, 3)
    );
    assert!(
        Bitmap::<OpaqueBuffer>::try_from_parts(
            Opaque {
                addr: 0,
                len: 1,
                _t: PhantomData
            },
            9,
            0
        )
        .is_err()
    );
}

#[test]
fn host_is_raw() {
    let v: Vec<u8> = vec![1, 2];
    assert_eq!(RawSlice::<u8>::memory_space(&v), MemorySpace::CPU);
    let _ = Bitmap::<VecBuffer>::from_parts(v, 16, 0);
}
