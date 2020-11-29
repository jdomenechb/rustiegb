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
    fn read_8(&self, position: u16) -> u8 {
        self.data.read_8(position)
    }

    fn read_16(&self, position: u16) -> u16 {
        self.data.read_16(position)
    }
}

impl Default for ReadOnlyMemorySector {
    fn default() -> Self {
        Self {
            data: MemorySector::new(0x8000),
        }
    }
}
