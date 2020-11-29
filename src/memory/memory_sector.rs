use crate::math::{two_bytes_to_word, word_to_two_bytes};

pub trait ReadMemory {
    fn read_byte(&self, position: u16) -> u8;
    fn read_word(&self, position: u16) -> u16;
}

pub trait WriteMemory {
    fn write_byte(&mut self, position: u16, value: u8);
    fn write_word(&mut self, position: u16, value: u16);
}

pub struct MemorySector {
    data: Vec<u8>,
}

impl MemorySector {
    pub fn with_size(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    pub fn with_data(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl ReadMemory for MemorySector {
    fn read_byte(&self, position: u16) -> u8 {
        return self.data[position as usize];
    }

    fn read_word(&self, position: u16) -> u16 {
        let position = position as usize;

        two_bytes_to_word(self.data[position + 1], self.data[position])
    }
}

impl WriteMemory for MemorySector {
    fn write_byte(&mut self, position: u16, value: u8) {
        self.data[position as usize] = value;
    }

    fn write_word(&mut self, position: u16, value: u16) {
        let position = position as usize;

        let bytes = word_to_two_bytes(value);

        self.data[position] = bytes.1;
        self.data[position + 1] = bytes.0;
    }
}
