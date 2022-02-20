use std::borrow::Borrow;

pub(crate) struct BitPacked<I, T>
where
    I: Iterator<Item = T>,
    T: Borrow<bool>,
{
    iter: I,
}

impl<I, T> Iterator for BitPacked<I, T>
where
    I: Iterator<Item = T>,
    T: Borrow<bool>,
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        // todo(mb): figure out why a map closure does not get inlined here
        // self.iter.next().map(|next| { ... });
        if let Some(next) = self.iter.next() {
            let mut byte = if *next.borrow() { 1 } else { 0 };
            for bit_position in 1u8..8 {
                if let Some(x) = self.iter.next() {
                    if *x.borrow() {
                        byte |= 1 << bit_position;
                    }
                }
            }
            Some(byte)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        (
            lower / 8 + (lower % 8 != 0) as usize,
            upper.map(|upper| upper / 8 + (upper % 8 != 0) as usize),
        )
    }
}

pub(crate) trait BitPackedExt<T>
where
    Self: Iterator<Item = T>,
    T: Borrow<bool>,
{
    fn bit_packed(self) -> BitPacked<Self, T>
    where
        Self: Sized,
    {
        BitPacked { iter: self }
    }
}

impl<I, T> BitPackedExt<T> for I
where
    I: Iterator<Item = T>,
    T: Borrow<bool>,
{
}

pub struct BitUnpacked<I, T>
where
    I: Iterator<Item = T>,
    T: Borrow<u8>,
{
    iter: I,
    byte: Option<T>,
    mask: u8,
}

impl<I, T> Iterator for BitUnpacked<I, T>
where
    I: Iterator<Item = T>,
    T: Borrow<u8> + Copy,
{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(byte) = self.byte.or_else(|| {
            self.byte = self.iter.next();
            self.byte
        }) {
            let next = byte.borrow() & self.mask != 0;
            self.mask = self.mask.rotate_left(1);
            if self.mask == 0x01 {
                self.byte = None;
            }
            Some(next)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        (lower * 8, upper.map(|upper| upper * 8))
    }
}

pub trait BitUnpackedExt<T>
where
    Self: Iterator<Item = T>,
    T: Borrow<u8>,
{
    fn bit_unpacked(self) -> BitUnpacked<Self, T>
    where
        Self: Sized,
    {
        BitUnpacked {
            iter: self,
            byte: None,
            mask: 0x01,
        }
    }
}

impl<I, T> BitUnpackedExt<T> for I
where
    I: Iterator<Item = T>,
    T: Borrow<u8>,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpack() {
        let input = [u8::MAX, 1];
        assert_eq!(
            input.iter().bit_unpacked().collect::<Vec<_>>(),
            vec![
                true, true, true, true, true, true, true, true, true, false, false, false, false,
                false, false, false
            ]
        );
    }

    #[test]
    fn unpack_size_hint() {
        let input = [u8::MAX, 1, 2, 3];
        assert_eq!(
            input.iter().bit_unpacked().size_hint(),
            (input.len() * 8, Some(input.len() * 8))
        );
    }
}
