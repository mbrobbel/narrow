//! Interop with [`arrow-rs`] null buffer.

use crate::Length;

impl Length for arrow_buffer::NullBuffer {
    fn len(&self) -> usize {
        arrow_buffer::NullBuffer::len(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: [bool; 4] = [true, true, false, true];

    #[test]
    fn length() {
        let null_buffer = INPUT.into_iter().collect::<arrow_buffer::NullBuffer>();
        assert_eq!(Length::len(&null_buffer), INPUT.len());
    }
}
