use crate::{
    bitmap::ValidityBitmap,
    buffer::{Buffer, BufferRef},
    offset::{self, Offset},
    validity::Validity,
    Length,
};

use super::{Array, ArrayType};

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

impl<const NULLABLE: bool, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> Array
    for StringArray<NULLABLE, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<NULLABLE>,
    BitmapBuffer: Buffer<u8>,
{
    type Item = String;
}

impl ArrayType for &str {
    type Array<
        DataBuffer: Buffer<Self::Primitive>,
        BitmapBuffer: Buffer<u8>,
        OffsetElement: offset::OffsetElement,
        OffsetBuffer: Buffer<OffsetElement>,
    > = StringArray<false, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>;
    type Primitive = u8;
    type RefItem<'a> = &'a str;
}

impl ArrayType for Option<&str> {
    type Array<
        DataBuffer: Buffer<Self::Primitive>,
        BitmapBuffer: Buffer<u8>,
        OffsetElement: offset::OffsetElement,
        OffsetBuffer: Buffer<OffsetElement>,
    > = StringArray<true, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>;
    type Primitive = u8;
    type RefItem<'a> = Option<&'a str>;
}

impl ArrayType for String {
    type Array<
        DataBuffer: Buffer<Self::Primitive>,
        BitmapBuffer: Buffer<u8>,
        OffsetElement: offset::OffsetElement,
        OffsetBuffer: Buffer<OffsetElement>,
    > = StringArray<false, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>;
    type Primitive = u8;
    type RefItem<'a> = &'a str;
}

impl ArrayType for Option<String> {
    type Array<
        DataBuffer: Buffer<Self::Primitive>,
        BitmapBuffer: Buffer<u8>,
        OffsetElement: offset::OffsetElement,
        OffsetBuffer: Buffer<OffsetElement>,
    > = StringArray<true, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>;
    type Primitive = u8;
    type RefItem<'a> = Option<&'a str>;
}

impl<'a, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> FromIterator<&'a str>
    for StringArray<false, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<false>,
    BitmapBuffer: Buffer<u8>,
    Offset<false, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>: FromIterator<&'a [u8]>,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        Self(iter.into_iter().map(|x| x.as_bytes()).collect())
    }
}

impl<DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> FromIterator<String>
    for StringArray<false, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<false>,
    BitmapBuffer: Buffer<u8>,
    Offset<false, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>: FromIterator<Vec<u8>>,
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

impl<'a, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> FromIterator<Option<&'a str>>
    for StringArray<true, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<true>,
    BitmapBuffer: Buffer<u8>,
    Offset<true, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>:
        FromIterator<Option<&'a [u8]>>,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = Option<&'a str>>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|x| x.map(|string_ref| string_ref.as_bytes()))
                .collect(),
        )
    }
}

impl<DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> FromIterator<Option<String>>
    for StringArray<true, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<true>,
    BitmapBuffer: Buffer<u8>,
    Offset<true, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>:
        FromIterator<Option<Vec<u8>>>,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = Option<String>>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|x| x.map(|string| string.into_bytes()))
                .collect(),
        )
    }
}

impl<DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer> ValidityBitmap
    for StringArray<true, DataBuffer, OffsetElement, OffsetBuffer, BitmapBuffer>
where
    DataBuffer: Buffer<u8>,
    OffsetElement: offset::OffsetElement,
    OffsetBuffer: Buffer<OffsetElement> + Validity<true>,
    BitmapBuffer: Buffer<u8>,
{
    type Buffer = BitmapBuffer;

    fn validity_bitmap(&self) -> &crate::bitmap::Bitmap<Self::Buffer> {
        self.0.validity_bitmap()
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
        let array = input.into_iter().collect::<LargeUtf8Array>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.buffer_ref(), b"1234567890".as_bytes());
    }

    #[test]
    fn size_of() {}

    #[test]
    fn nullable_string_array() {
        // &str
        let input = [Some("1"), None, Some("23")]
            .into_iter()
            .map(|string_ref| string_ref.map(ToString::to_string))
            .collect::<Vec<_>>();
        let array = input.into_iter().collect::<Utf8Array<true>>();

        assert_eq!(array.null_count(), 1);
        assert_eq!(array.buffer_ref(), b"123".as_bytes());

        // String
        let input = [Some("1"), Some("23"), None, Some("67")];
        let array = input.into_iter().collect::<Utf8Array<true>>();

        assert_eq!(array.null_count(), 1);
        assert_eq!(array.buffer_ref(), b"12367".as_bytes());
    }
}
