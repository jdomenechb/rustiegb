use crate::{Byte, Word};

pub trait ReadMemory {
    fn read_byte(&self, position: Word) -> Byte;
}

pub trait WriteMemory {
    fn write_byte(&mut self, position: Word, value: Byte);
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
}

impl ReadMemory for MemorySector {
    fn read_byte(&self, position: Word) -> Byte {
        self.data[position as usize]
    }
}

impl WriteMemory for MemorySector {
    fn write_byte(&mut self, position: Word, value: Byte) {
        self.data[position as usize] = value;
    }
}
