use crate::{Byte, Word};

#[readonly::make]
#[derive(Debug)]
pub struct NRxx {
    pub value: Byte,
    used_bits: Byte,
    read_ored_bits: Byte,
}

impl NRxx {
    const DEFAULT_USED_BITS: Byte = 0b1111_1111;
    const DEFAULT_READ_ORED_BITS: Byte = 0x00;

    pub fn new(default: Byte) -> Self {
        Self {
            value: default,
            used_bits: Self::DEFAULT_USED_BITS,
            read_ored_bits: Self::DEFAULT_READ_ORED_BITS,
        }
    }

    pub fn new_with_used_bits(default: Byte, used_bits: Byte) -> Self {
        Self {
            value: default,
            used_bits,
            read_ored_bits: Self::DEFAULT_READ_ORED_BITS,
        }
    }

    pub fn new_with_read_ored_bits(default: Byte, read_ored_bits: Byte) -> Self {
        Self {
            value: default,
            used_bits: Self::DEFAULT_USED_BITS,
            read_ored_bits,
        }
    }

    pub fn new_with(default: Byte, used_bits: Byte, read_ored_bits: Byte) -> Self {
        Self {
            value: default,
            used_bits,
            read_ored_bits,
        }
    }

    pub fn read(&self) -> Byte {
        self.value | self.read_ored_bits
    }

    pub fn reset(&mut self) {
        self.value = !self.used_bits;
    }

    pub fn update(&mut self, value: Byte) {
        self.value = (value & self.used_bits) | !self.used_bits;
    }

    pub fn update_low_frequency(&mut self, frequency: Word) {
        self.value = (frequency & 0xFF) as Byte
    }

    pub fn update_high_frequency(&mut self, frequency: Word) {
        self.value = (self.value & 0b11111000) | ((frequency >> 8) & 0b111) as Byte
    }
}
