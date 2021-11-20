use crate::{Byte, Word};

#[derive(Default, Clone)]
pub struct Dma {
    value: Byte,
    remaining_cycles: Byte,
}

impl Dma {
    pub fn step(&mut self, cycles: u8) -> bool {
        if self.remaining_cycles <= 0 {
            return false;
        }

        self.remaining_cycles -= cycles;

        if self.remaining_cycles <= 0 {
            self.remaining_cycles = 0;
            return true;
        }

        return false;
    }
}

impl From<Byte> for Dma {
    fn from(value: Byte) -> Self {
        Self {
            value,
            remaining_cycles: 160,
        }
    }
}

impl From<&Dma> for Byte {
    fn from(original: &Dma) -> Self {
        original.value
    }
}

impl From<&Dma> for Word {
    fn from(original: &Dma) -> Self {
        (original.value as Word) << 8 & 0xFF00
    }
}
