use crate::memory::memory_sector::{MemorySector, ReadMemory, WriteMemory};
use crate::{Byte, Word};

pub struct VideoRam8kMemorySector {
    data: MemorySector,
}

impl ReadMemory for VideoRam8kMemorySector {
    fn read_byte(&self, position: Word) -> Byte {
        self.data.read_byte(position)
    }
}

impl WriteMemory for VideoRam8kMemorySector {
    fn write_byte(&mut self, position: Word, value: Byte) {
        self.data.write_byte(position, value);
    }
}

impl Default for VideoRam8kMemorySector {
    fn default() -> Self {
        Self {
            data: MemorySector::with_size(0x2000),
        }
    }
}
