use crate::{Byte, Word};

pub fn two_bytes_to_word(h: Byte, l: Byte) -> Word {
    let result = h as Word;
    (result << 8) + (l as Word)
}

pub fn word_to_two_bytes(value: Word) -> (Byte, Byte) {
    let low = value as Byte;
    let high = (value >> 8) as Byte;

    (high, low)
}
