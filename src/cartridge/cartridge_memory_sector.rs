use crate::math::{two_bytes_to_word, word_to_two_bytes};
use crate::{Byte, Word};

pub trait ReadCartridgeMemory {
    fn read_byte(&self, position: usize) -> Byte;
    fn read_word(&self, position: usize) -> Word;
}

pub trait WriteCartridgeMemory {
    fn write_byte(&mut self, position: usize, value: Byte);
    fn write_word(&mut self, position: usize, value: Word);
}

pub struct CartridgeMemorySector {
    data: Vec<Byte>,
}

impl CartridgeMemorySector {
    pub fn of_size(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    pub fn from_data(data: Vec<Byte>) -> Self {
        Self { data }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}

impl ReadCartridgeMemory for CartridgeMemorySector {
    fn read_byte(&self, position: usize) -> Byte {
        self.data[position as usize]
    }

    fn read_word(&self, position: usize) -> Word {
        let position = position as usize;

        two_bytes_to_word(self.data[position + 1], self.data[position])
    }
}

impl WriteCartridgeMemory for CartridgeMemorySector {
    fn write_byte(&mut self, position: usize, value: Byte) {
        self.data[position as usize] = value;
    }

    fn write_word(&mut self, position: usize, value: Word) {
        let position = position as usize;

        let bytes = word_to_two_bytes(value);

        self.data[position] = bytes.1;
        self.data[position + 1] = bytes.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_write_out_of_bonds_panics() {
        let mut cartridge_sector = CartridgeMemorySector::of_size(4);
        cartridge_sector.write_byte(4, 0xFF);

        assert_eq!(cartridge_sector.data.len(), 0);
    }
}
