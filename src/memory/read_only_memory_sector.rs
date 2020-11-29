use crate::memory::memory_sector::{MemorySector, ReadMemory};

pub struct ReadOnlyMemorySector {
    pub data: MemorySector,
}

impl ReadOnlyMemorySector {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: MemorySector::with_data(data),
        }
    }
}

impl ReadMemory for ReadOnlyMemorySector {
    fn read_byte(&self, position: u16) -> u8 {
        self.data.read_byte(position)
    }

    fn read_word(&self, position: u16) -> u16 {
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
