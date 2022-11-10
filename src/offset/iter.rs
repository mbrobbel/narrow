use std::iter::{self, Chain, Once, Zip};

use super::OffsetElement;
use crate::Length;

pub struct ScanOffsets<I, T>
where
    I: Iterator<Item = usize>,
    T: OffsetElement,
{
    iter: I,
    state: T,
}

impl<I, T> Iterator for ScanOffsets<I, T>
where
    I: Iterator<Item = usize>,
    T: OffsetElement,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| {
            self.state += T::try_from(item).unwrap();
            self.state
        })
    }
}

pub trait ScanOffsetsExt<T>
where
    T: OffsetElement,
{
    fn scan_offsets(self) -> Chain<Once<T>, ScanOffsets<Self, T>>
    where
        Self: Sized + Iterator<Item = usize>,
    {
        iter::once(T::default()).chain(ScanOffsets {
            iter: self,
            state: T::default(),
        })
    }
}

impl<I, T> ScanOffsetsExt<T> for I
where
    I: Iterator<Item = usize>,
    T: OffsetElement,
{
}

pub struct Offsets<I, T, U>
where
    I: Iterator<Item = T>,
    T: Length,
    U: OffsetElement,
{
    iter: I,
    state: U,
}

impl<I, T, U> Iterator for Offsets<I, T, U>
where
    I: Iterator<Item = T>,
    T: Length,
    U: OffsetElement,
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| {
            self.state += U::try_from(item.len()).unwrap();
            self.state
        })
    }
}

pub struct LengthIter<I, T>
where
    I: Iterator<Item = usize>,
    T: OffsetElement,
{
    iter: I,
    state: T,
}

impl<I, T> Iterator for LengthIter<I, T>
where
    I: Iterator<Item = usize>,
    T: OffsetElement,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| {
            self.state += T::try_from(item).unwrap();
            self.state
        })
    }
}

pub trait LengthExt<T>
where
    Self: Iterator<Item = usize>,
    T: OffsetElement,
{
    fn len_to_offsets(self) -> Chain<Once<T>, LengthIter<Self, T>>
    where
        Self: Sized,
    {
        iter::once(T::default()).chain(LengthIter {
            iter: self,
            state: T::default(),
        })
    }
}

impl<I, T> LengthExt<T> for I
where
    I: Iterator<Item = usize>,
    T: OffsetElement,
{
}

#[allow(unused)]
pub type WithOffsets<T, U, V> = Zip<Offsets<V, T, U>, V>;

pub trait OffsetsExt<T, U>
where
    Self: Iterator<Item = T>,
    T: Length,
    U: OffsetElement,
{
    fn offsets(self) -> Chain<Once<U>, Offsets<Self, T, U>>
    where
        Self: Sized,
    {
        iter::once(U::default()).chain(Offsets {
            iter: self,
            state: U::default(),
        })
    }

    fn with_offsets(self) -> Chain<Once<(U, T)>, WithOffsets<T, U, Self>>
    where
        T: Default,
        Self: Clone + Sized,
    {
        // TODO(mbrobbel): add first element here and use default for last element
        iter::once((U::default(), T::default())).chain(
            Offsets {
                iter: self.clone(),
                state: U::default(),
            }
            .zip(self),
        )
    }
}

impl<I, T, U> OffsetsExt<T, U> for I
where
    I: Iterator<Item = T>,
    T: Length,
    U: OffsetElement,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
    struct Count(usize);
    impl Length for Count {
        fn len(&self) -> usize {
            self.0
        }
    }

    #[test]
    fn offsets() {
        let input = [Count(1), Count(2), Count(3), Count(4)];
        assert_eq!(
            input.iter().offsets().collect::<Vec<i32>>(),
            [i32::default(), 1, 3, 6, 10]
        );
    }

    #[test]
    fn with_offsets() {
        let input = [Count(1), Count(2), Count(3), Count(4)];
        assert_eq!(
            input
                .into_iter()
                .with_offsets()
                .collect::<Vec<(i32, Count)>>(),
            vec![
                (i32::default(), Count(0)),
                (1, Count(1)),
                (3, Count(2)),
                (6, Count(3)),
                (10, Count(4))
            ]
        );
    }
}
