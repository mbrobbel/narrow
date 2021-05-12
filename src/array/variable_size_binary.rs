use crate::{Array, Buffer, Data, Nullable, Offset, OffsetType, ALIGNMENT};
use std::{iter::FromIterator, ops::Index};

/// Array with variable-sized binary data.
#[derive(Debug)]
pub struct VariableSizeBinaryArray<T, const N: bool>
where
    T: OffsetType,
{
    data: Buffer<u8, ALIGNMENT>,
    offset: Offset<T, N>,
}

impl<T> Array for VariableSizeBinaryArray<T, false>
where
    T: OffsetType,
{
    type Data = Buffer<T, ALIGNMENT>;

    fn data(&self) -> &Self::Data {
        &self.offset
    }

    fn len(&self) -> usize {
        self.data().len() - 1
    }
}

impl<T> Array for VariableSizeBinaryArray<T, true>
where
    T: OffsetType,
{
    type Data = Nullable<Buffer<T, ALIGNMENT>>;

    fn data(&self) -> &Self::Data {
        &self.offset
    }
}

impl<T, U> FromIterator<U> for VariableSizeBinaryArray<T, false>
where
    T: OffsetType,
    U: AsRef<[u8]>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = U>,
    {
        let mut data = Vec::default();

        let offset = iter
            .into_iter()
            .inspect(|x| {
                data.extend_from_slice(x.as_ref());
            })
            .map(|x| T::try_from(x.as_ref().len()).unwrap())
            .collect::<Offset<T, false>>();

        Self {
            data: data.into(),
            offset,
        }
    }
}

impl<T, U> FromIterator<Option<U>> for VariableSizeBinaryArray<T, true>
where
    T: OffsetType,
    U: AsRef<[u8]>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Option<U>>,
    {
        let mut data = Vec::default();

        let offset = iter
            .into_iter()
            .inspect(|opt| {
                if let Some(slice) = opt {
                    data.extend_from_slice(slice.as_ref());
                }
            })
            .map(|opt| opt.map(|slice| T::try_from(slice.as_ref().len()).unwrap()))
            .collect();

        Self {
            data: data.into(),
            offset,
        }
    }
}

impl<T> Index<usize> for VariableSizeBinaryArray<T, false>
where
    T: OffsetType,
{
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        // todo(mb): assert conditions
        let start = self.offset[index].try_into().unwrap();
        let end = self.offset[index + 1].try_into().unwrap();
        &self.data[start..end]
    }
}

/// Array with variable sized binary data. Uses [i32] offsets.
pub type BinaryArray<const N: bool> = VariableSizeBinaryArray<i32, N>;

/// Array with variable sized binary data. Uses [i64] offsets.
pub type LargeBinaryArray<const N: bool> = VariableSizeBinaryArray<i64, N>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Array;

    #[test]
    fn from_iter() {
        let x = vec![1u8, 2, 3, 4, 5];
        let y = vec![1u8, 2, 3, 4];
        let z = vec![1u8, 2, 3];
        let vec = vec![&x[..], &y, &z];

        let array = vec.iter().collect::<BinaryArray<false>>();
        assert_eq!(array.len(), 3);
        assert_eq!(&array[0], &x[..]);
        assert_eq!(&array[1], &y[..]);
        assert_eq!(&array[2], &z[..]);

        let x = vec![1u8, 2, 3, 4, 5];
        let y = vec![1u8, 2, 3, 4];
        let vec = vec![Some(&x[..]), Some(&y), None, None, Some(&x), Some(&[])];
        let array = vec.into_iter().collect::<LargeBinaryArray<true>>();
        assert_eq!(array.len(), 6);
    }
}
