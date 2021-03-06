use crate::memory::memory_sector::{MemorySector, ReadMemory};
use crate::{Byte, Word};
use std::cmp::min;

pub struct ReadOnlyMemorySector {
    pub data: MemorySector,
}

impl ReadOnlyMemorySector {
    pub fn new(data: &Vec<Byte>) -> Self {
        Self {
            data: MemorySector::with_data((&data[0x0..min(0x8000, data.len())]).to_vec()),
        }
    }
}

impl ReadMemory for ReadOnlyMemorySector {
    fn read_byte(&self, position: Word) -> Byte {
        self.data.read_byte(position)
    }

    fn read_word(&self, position: Word) -> Word {
        self.data.read_word(position)
    }
}

impl Default for ReadOnlyMemorySector {
    fn default() -> Self {
        Self {
            data: MemorySector::with_size(0x8000),
        }
    }
}
