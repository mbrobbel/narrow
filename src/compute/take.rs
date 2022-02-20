use crate::{FixedSizePrimitiveArray, Primitive};

pub trait Take<T, const N: bool>
where
    T: Primitive,
{
    type Output;
    fn take(&self, indices: &FixedSizePrimitiveArray<T, N>) -> Self::Output;
}

impl<T, U> Take<T, false> for FixedSizePrimitiveArray<U, false>
where
    T: Primitive + TryInto<usize>,
    T::Error: std::fmt::Debug,
    U: Primitive,
{
    type Output = FixedSizePrimitiveArray<U, false>;

    fn take(&self, indices: &FixedSizePrimitiveArray<T, false>) -> Self::Output {
        indices
            .into_iter()
            .map(|&index| &self[index.try_into().unwrap()])
            .collect()
    }
}

impl<T, U> Take<T, true> for FixedSizePrimitiveArray<U, false>
where
    T: Primitive + TryInto<usize>,
    T::Error: std::fmt::Debug,
    U: Primitive,
{
    type Output = FixedSizePrimitiveArray<U, true>;

    fn take(&self, indices: &FixedSizePrimitiveArray<T, true>) -> Self::Output {
        indices
            .into_iter()
            .map(|opt| opt.map(|&index| self[index.try_into().unwrap()]))
            .collect()
    }
}

impl<T, U> Take<T, false> for FixedSizePrimitiveArray<U, true>
where
    T: Primitive + TryInto<usize>,
    T::Error: std::fmt::Debug,
    U: Primitive,
{
    type Output = FixedSizePrimitiveArray<U, true>;

    fn take(&self, indices: &FixedSizePrimitiveArray<T, false>) -> Self::Output {
        indices
            .into_iter()
            .map(|&index| self.get(index.try_into().unwrap()))
            .collect()
    }
}

impl<T, U> Take<T, true> for FixedSizePrimitiveArray<U, true>
where
    T: Primitive + TryInto<usize>,
    T::Error: std::fmt::Debug,
    U: Primitive,
{
    type Output = FixedSizePrimitiveArray<U, true>;

    fn take(&self, indices: &FixedSizePrimitiveArray<T, true>) -> Self::Output {
        indices
            .into_iter()
            .map(|opt| opt.and_then(|&index| self.get(index.try_into().unwrap())))
            .collect()
    }
}

pub fn take(
    input: &FixedSizePrimitiveArray<u32, true>,
    indices: &FixedSizePrimitiveArray<u32, true>,
) -> FixedSizePrimitiveArray<u32, true> {
    input.take(indices)
}

#[cfg(test)]
mod tests {
    use crate::Uint8Array;

    use super::*;

    #[test]
    fn fixed_size_primitive_array() {
        // all valid
        let array = [1, 2, 3, 4, 5].into_iter().collect::<Uint8Array<false>>();
        let indices = [3, 1].into_iter().collect::<Uint8Array<false>>();
        let take = array.take(&indices);
        assert_eq!(take, [4, 2].into_iter().collect());

        // indices nullable
        let indices = [None, Some(3), Some(1), None, None]
            .into_iter()
            .collect::<Uint8Array>();
        let take = array.take(&indices);
        assert_eq!(
            take,
            [None, Some(4), Some(2), None, None].into_iter().collect()
        );

        // values nullable
        let array = [Some(1), Some(2), None, Some(4), None]
            .into_iter()
            .collect::<Uint8Array>();
        let indices = [0, 1, 2, 2, 3].into_iter().collect::<Uint8Array<false>>();
        let take = array.take(&indices);
        assert_eq!(
            take,
            [Some(1), Some(2), None, None, Some(4)]
                .into_iter()
                .collect()
        );

        // both nullable
        let array = [Some(1), Some(2), None, Some(4), None]
            .into_iter()
            .collect::<Uint8Array>();
        let indices = [None, Some(3), Some(1), None, None]
            .into_iter()
            .collect::<Uint8Array>();
        let take = array.take(&indices);
        assert_eq!(
            take,
            [None, Some(4), Some(2), None, None].into_iter().collect()
        );
    }
}
