use crate::{
    array,
    buffer::{Buffer, BufferExtend, BufferTake},
    Length, Primitive,
};
use std::{
    iter::{Peekable, Repeat, Take, Zip},
    marker::PhantomData,
    num::TryFromIntError,
    ops::{Add, AddAssign, Index, Sub},
};

pub struct RunEndEncoded<Array, RunEndElement = i16, RunEndBuffer = Vec<RunEndElement>>
where
    Array: array::Array,
    RunEndElement: self::RunEndElement,
    RunEndBuffer: Buffer<RunEndElement>,
{
    run_ends: RunEndBuffer,
    values: Array,
    _element_ty: PhantomData<fn() -> RunEndElement>,
}

impl<Array, RunEndElement, RunEndBuffer> RunEndEncoded<Array, RunEndElement, RunEndBuffer>
where
    Array: array::Array + Index<usize, Output = Array::Item>,
    RunEndElement: self::RunEndElement,
    RunEndBuffer: Buffer<RunEndElement>,
{
    /// Returns the length of the run at the given index. Returns [Option::None] if the index is out of bounds.
    pub fn run_len(&self, index: usize) -> Option<usize> {
        self.run_ends
            .borrow()
            .get(index)
            .map(|len| {
                *len - index
                    .checked_sub(1)
                    .and_then(|index| self.run_ends.borrow().get(index))
                    .copied()
                    .unwrap_or_default()
            })
            .map(|len| len.try_into().unwrap())
    }

    pub fn run_value(&self, index: usize) -> Option<&Array::Item> {
        // binary search
        if index <= self.len() {
            // or add one to index? readability?
            let index = RunEndElement::try_from(index).unwrap();
            Some(
                match self.run_ends.borrow().binary_search_by(|probe| {
                    (*probe - RunEndElement::try_from(1).unwrap()).cmp(&index)
                }) {
                    Ok(index) | Err(index) => self.values.index(index),
                },
            )
        } else {
            None
        }
    }
}

impl<Array, RunEndElement, RunEndBuffer> Length
    for RunEndEncoded<Array, RunEndElement, RunEndBuffer>
where
    Array: array::Array,
    RunEndElement: self::RunEndElement,
    RunEndBuffer: Buffer<RunEndElement>,
{
    fn len(&self) -> usize {
        self.run_ends
            .borrow()
            .last()
            .map(|&len| len.try_into().unwrap())
            .unwrap_or_default()
    }
}

pub trait RunEndElement:
    Primitive
    + Add
    + Ord
    + AddAssign
    + TryFrom<usize, Error = TryFromIntError>
    + TryInto<usize, Error = TryFromIntError>
    + Sub<Output = Self>
    + sealed::Sealed
{
    fn abs_diff(&self, other: Self) -> usize;
    fn one() -> Self;
}

mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for T where T: super::RunEndElement {}
}

impl RunEndElement for i16 {
    fn abs_diff(&self, other: Self) -> usize {
        usize::try_from(i16::abs_diff(*self, other)).unwrap()
    }

    fn one() -> Self {
        1
    }
}
impl RunEndElement for i32 {
    fn abs_diff(&self, other: Self) -> usize {
        usize::try_from(i32::abs_diff(*self, other)).unwrap()
    }

    fn one() -> Self {
        1
    }
}
impl RunEndElement for i64 {
    fn abs_diff(&self, other: Self) -> usize {
        usize::try_from(i64::abs_diff(*self, other)).unwrap()
    }

    fn one() -> Self {
        1
    }
}

pub struct RunEnds<I, RunEndElement>
where
    I: Iterator,
{
    len: RunEndElement,
    iter: Peekable<I>,
}

impl<I, RunEndElement> Iterator for RunEnds<I, RunEndElement>
where
    I: Iterator,
    <I as Iterator>::Item: PartialEq,
    RunEndElement: self::RunEndElement,
{
    type Item = (RunEndElement, <I as Iterator>::Item);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| {
            let mut run = 1;
            while self.iter.next_if_eq(&item).is_some() {
                run += 1;
            }
            self.len += RunEndElement::try_from(run).unwrap();
            (self.len, item)
        })
    }

    // todo: size_hint
}

pub trait RunEndsExt<T, RunEndElement>
where
    Self: Iterator<Item = T>,
    RunEndElement: self::RunEndElement,
{
    fn run_end_encoded(self) -> RunEnds<Self, RunEndElement>
    where
        Self: Sized,
    {
        RunEnds {
            len: RunEndElement::default(),
            iter: self.peekable(),
        }
    }
}

impl<I, T, RunEndElement> RunEndsExt<T, RunEndElement> for I
where
    I: Iterator<Item = T>,
    RunEndElement: self::RunEndElement,
{
}

impl<Array, T, RunEndElement, RunEndBuffer> FromIterator<T>
    for RunEndEncoded<Array, RunEndElement, RunEndBuffer>
where
    Array: array::Array + FromIterator<T>,
    T: PartialEq,
    RunEndElement: self::RunEndElement,
    RunEndBuffer: Default + BufferExtend<RunEndElement>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut run_ends = RunEndBuffer::default();
        let values = iter
            .into_iter()
            .run_end_encoded()
            .map(|(run, item)| {
                run_ends.extend(Some(run));
                item
            })
            .collect();
        Self {
            run_ends,
            values,
            _element_ty: PhantomData,
        }
    }
}

pub struct RunEndsIter<I, J, RunEndElement>
where
    I: Iterator,
    J: Iterator<Item = RunEndElement>,
{
    end: RunEndElement,
    run: Option<Take<Repeat<I::Item>>>,
    iter: Zip<I, J>,
}

impl<I, J, RunEndElement> Iterator for RunEndsIter<I, J, RunEndElement>
where
    I: Iterator,
    I::Item: Clone,
    J: Iterator<Item = RunEndElement>,
    RunEndElement: self::RunEndElement,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.run
            .as_mut()
            .and_then(|run| run.next())
            .or_else(|| {
                self.run = self.iter.next().map(|(item, run)| {
                    std::iter::repeat(item).take((run - self.end).try_into().unwrap())
                });
                self.run.as_mut().and_then(|run| run.next())
            })
            .map(|item| {
                self.end += RunEndElement::one();
                item
            })
    }
}

impl<Array, RunEndElement, RunEndBuffer> IntoIterator
    for RunEndEncoded<Array, RunEndElement, RunEndBuffer>
where
    Array: array::Array + IntoIterator,
    <Array as IntoIterator>::Item: Clone,
    RunEndElement: self::RunEndElement,
    RunEndBuffer: BufferTake<RunEndElement>,
{
    type Item = <Array as IntoIterator>::Item;
    type IntoIter = RunEndsIter<Array::IntoIter, RunEndBuffer::IntoIter, RunEndElement>;

    fn into_iter(self) -> Self::IntoIter {
        RunEndsIter {
            end: RunEndElement::default(),
            run: None,
            iter: self.values.into_iter().zip(self.run_ends.into_iter()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        array::{fixed_size_primitive::Int8Array, string::StringArray},
        bitmap::ValidityBitmap,
        buffer::BufferRef,
    };

    #[test]
    fn from_iter() {
        let input = [1i8, 1, 1, 1, 0, 0, 2];
        let array = RunEndEncoded::<Int8Array>::from_iter(input);
        assert_eq!(array.len(), 7);
        assert_eq!(array.run_len(0), Some(4));
        assert_eq!(array.run_len(1), Some(2));
        assert_eq!(array.run_len(2), Some(1));
        assert!(array.run_len(3).is_none());

        assert_eq!(array.run_value(0), Some(&1i8));
        assert_eq!(array.run_value(1), Some(&1i8));
        assert_eq!(array.run_value(2), Some(&1i8));
        assert_eq!(array.run_value(3), Some(&1i8));
        assert_eq!(array.run_value(4), Some(&0i8));
        assert_eq!(array.run_value(5), Some(&0i8));
        assert_eq!(array.run_value(6), Some(&2i8));

        let input = [Some(1i8), Some(1), Some(1), Some(1), None, None, Some(2)];
        let array = RunEndEncoded::<Int8Array<true>>::from_iter(input);
        assert_eq!(array.len(), 7);
        assert_eq!(array.values.null_count(), 1);

        let input = ["hello", "world", "world", "world"];
        let array = input.into_iter().collect::<RunEndEncoded<StringArray>>();
        assert_eq!(array.len(), 4);
        assert_eq!(array.values.len(), 2);
        assert_eq!(array.values.buffer_ref().len(), "helloworld".len());
    }

    #[test]
    fn into_iter() {
        let input = [1i8, 1, 1, 1, 0, 0, 2];
        let array = input.into_iter().collect::<RunEndEncoded<Int8Array>>();
        let output = array.into_iter().collect::<Vec<_>>();
        assert_eq!(input, output.as_slice());

        let input = [Some(1), None, Some(1), Some(2), Some(2), None, None];
        let array = input
            .into_iter()
            .collect::<RunEndEncoded<Int8Array<true>>>();
        let output = array.into_iter().collect::<Vec<Option<i8>>>();
        assert_eq!(input, output.as_slice());
    }

    #[test]
    fn from_array() {
        let input = [1i8, 2, 3, 4, 5, 5, 5, 5, 5];
        let array = input.into_iter().collect::<Int8Array>();
        let ree_array = array.into_iter().collect::<RunEndEncoded<Int8Array>>();
        assert_eq!(ree_array.len(), input.len());
    }

    #[test]
    fn one_run() {
        let input = [0i8; 1234];
        let array = RunEndEncoded::<Int8Array>::from_iter(input);
        assert_eq!(array.len(), 1234);
        assert_eq!(array.values.len(), 1);
    }

    #[test]
    #[should_panic]
    fn run_end_overflow() {
        let input = [0i8; i16::MAX as usize + 1];
        let _array = RunEndEncoded::<Int8Array>::from_iter(input);
    }
}
