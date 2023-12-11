//! Interop with [`arrow-rs`] buffer types.

pub mod boolean_buffer;
pub mod buffer_builder;
pub mod null_buffer;
pub mod offset_buffer;
pub mod scalar_buffer;

// /// A [`BufferType`] implementation for &'a arrow Buffer.
// ///
// /// Stores items `T` in an arrow `&Buffer`.
// #[derive(Clone, Copy, Debug)]
// pub struct ArrowRefBuffer<'a>(PhantomData<&'a ()>);

// impl<'a> BufferType for ArrowRefBuffer<'a> {
//     type Buffer<T: FixedSize> = &'a [T];
// }

// impl<T: FixedSize> Buffer<T> for &arrow_buffer::Buffer {
//     fn as_slice(&self) -> &[T] {
//         self.typed_data()
//     }
// }

// impl Length for &arrow_buffer::Buffer {
//     fn len(&self) -> usize {
//         arrow_buffer::Buffer::len(self)
//     }
// }

// impl Index for &arrow_buffer::Buffer {
//     type Item<'a> = &'a u8
//     where
//         Self: 'a;

//     unsafe fn index_unchecked(&self, index: usize) -> Self::Item<'_> {
//         self.get_unchecked(index)
//     }
// }
