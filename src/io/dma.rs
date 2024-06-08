use crate::{Byte, Word};

#[derive(Default, Clone)]
pub struct Dma {
    value: Byte,
    remaining_cycles: Byte,
}

impl Dma {
    pub fn step(&mut self, cycles: u8) -> bool {
        if self.remaining_cycles == 0 {
            return false;
        }

        self.remaining_cycles = self.remaining_cycles.saturating_sub(cycles);

        if self.remaining_cycles == 0 {
            return true;
        }

        false
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
        let mut value = original.value;

        if value > 0xDF {
            value &= 0xDF;
        }

        (value as Word) << 8 & 0xFF00
    }
}
