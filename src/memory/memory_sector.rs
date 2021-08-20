use crate::math::{two_bytes_to_word, word_to_two_bytes};
use crate::{Byte, Word};

pub trait ReadMemory {
    fn read_byte(&self, position: Word) -> Byte;
    fn read_word(&self, position: Word) -> Word;
}

pub trait WriteMemory {
    fn write_byte(&mut self, position: Word, value: Byte);
    fn write_word(&mut self, position: Word, value: Word);
}

#[readonly::make]
pub struct MemorySector {
    pub data: Vec<Byte>,
}

impl MemorySector {
    pub fn with_size(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    pub fn with_data(data: Vec<Byte>) -> Self {
        Self { data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl ReadMemory for MemorySector {
    fn read_byte(&self, position: Word) -> Byte {
        return self.data[position as usize];
    }

    fn read_word(&self, position: Word) -> Word {
        let position = position as usize;

        two_bytes_to_word(self.data[position + 1], self.data[position])
    }
}

impl WriteMemory for MemorySector {
    fn write_byte(&mut self, position: Word, value: Byte) {
        self.data[position as usize] = value;
    }

    fn write_word(&mut self, position: Word, value: Word) {
        let position = position as usize;

        let bytes = word_to_two_bytes(value);

        self.data[position] = bytes.1;
        self.data[position + 1] = bytes.0;
    }
}
