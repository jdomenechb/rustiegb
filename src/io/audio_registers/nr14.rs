use crate::io::UpdatableRegister;
use crate::{Byte, Word};

#[readonly::make]
pub struct NR14 {
    pub value: Byte,
}

impl NR14 {
    pub fn update_frequency(&mut self, frequency: Word) {
        self.value = (self.value & 0b11111000) | ((frequency >> 8) & 0b111) as Byte
    }
}

impl UpdatableRegister for NR14 {
    fn update(&mut self, value: Byte) {
        self.value = value | 0b0011_1000;
    }
}

impl Default for NR14 {
    fn default() -> Self {
        Self { value: 0xBF }
    }
}
