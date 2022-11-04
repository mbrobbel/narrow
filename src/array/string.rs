use crate::{
    buffer::{Buffer, BufferRef},
    offset::{self, Offset},
    validity::Validity,
    Length,
};

pub struct StringArray<
    const NULLABLE: bool = false,
    DataBuffer = Vec<u8>,
    OffsetElement = i32,
    OffsetBuffer = Vec<OffsetElement>,
    BitmapBuffer = Vec<u8>,
>(Offset<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>)
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>;

pub type Utf8Array<
    const NULLABLE: bool = false,
    DataBuffer = Vec<u8>,
    OffsetBuffer = Vec<i32>,
    BitmapBuffer = Vec<u8>,
> = StringArray<NULLABLE, DataBuffer, i32, OffsetBuffer, BitmapBuffer>;

pub type LargeUtf8Array<
    const NULLABLE: bool = false,
    DataBuffer = Vec<u8>,
    OffsetBuffer = Vec<i64>,
    BitmapBuffer = Vec<u8>,
> = StringArray<NULLABLE, DataBuffer, i64, OffsetBuffer, BitmapBuffer>;

impl<'a, const NULLABLE: bool, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
    FromIterator<&'a str>
    for StringArray<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    Offset<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>: FromIterator<&'a [u8]>,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        Self(iter.into_iter().map(|x| x.as_bytes()).collect())
    }
}

impl<const NULLABLE: bool, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
    FromIterator<String>
    for StringArray<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    Offset<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>: FromIterator<Vec<u8>>,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self(iter.into_iter().map(|x| x.into_bytes()).collect())
    }
}

impl<const NULLABLE: bool, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> Length
    for StringArray<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    Offset<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>: Length,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<const NULLABLE: bool, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> BufferRef
    for StringArray<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
    Offset<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>: BufferRef,
{
    type Buffer = <Offset<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> as BufferRef>::Buffer;
    type Element = <Offset<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> as BufferRef>::Element;

    #[inline]
    fn buffer_ref(&self) -> &Self::Buffer {
        self.0.buffer_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_iter() {
        let input = ["1", "23", "456", "7890"];
        let array = input.into_iter().collect::<Utf8Array>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.buffer_ref(), b"1234567890".as_bytes());

        let input = ["1", "23", "456", "7890"]
            .into_iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>();
        let array = input.into_iter().collect::<Utf8Array>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.buffer_ref(), b"1234567890".as_bytes());
    }

    #[test]
    fn size_of() {}
}
