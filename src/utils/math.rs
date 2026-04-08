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

pub fn set_bit(base: &Byte, position: u8, value: bool) -> Byte {
    let mask = 1 << position;

    if value { base | mask } else { base & !mask }
}

pub fn is_bit_set(base: &Byte, position: u8) -> bool {
    let mask = 1 << position;

    base & mask == mask
}
