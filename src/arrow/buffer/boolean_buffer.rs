//! Interop with [`arrow-rs`] boolean buffer.

use crate::Length;

impl Length for arrow_buffer::BooleanBuffer {
    fn len(&self) -> usize {
        arrow_buffer::BooleanBuffer::len(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: [bool; 4] = [true, true, false, true];

    #[test]
    fn length() {
        let boolean_buffer = INPUT.into_iter().collect::<arrow_buffer::BooleanBuffer>();
        assert_eq!(Length::len(&boolean_buffer), INPUT.len());
    }
}
