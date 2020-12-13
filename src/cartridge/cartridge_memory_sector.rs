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
    pub fn with_size(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    pub fn with_data(data: Vec<Byte>) -> Self {
        Self { data }
    }
}

impl ReadCartridgeMemory for CartridgeMemorySector {
    fn read_byte(&self, position: usize) -> Byte {
        return self.data[position as usize];
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
