use crate::memory::memory_sector::{MemorySector, ReadMemory, WriteMemory};
use crate::{Byte, Word};

pub struct InternalRam8kMemorySector {
    data: MemorySector,
}

impl ReadMemory for InternalRam8kMemorySector {
    fn read_byte(&self, position: Word) -> Byte {
        self.data.read_byte(position)
    }
}

impl WriteMemory for InternalRam8kMemorySector {
    fn write_byte(&mut self, position: Word, value: Byte) {
        self.data.write_byte(position, value);
    }
}

impl Default for InternalRam8kMemorySector {
    fn default() -> Self {
        Self {
            data: MemorySector::with_size(0x2000),
        }
    }
}
