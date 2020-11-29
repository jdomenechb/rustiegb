use crate::memory::memory_sector::{MemorySector, ReadMemory, WriteMemory};
use crate::{Byte, Word};

pub struct InternalRamMemorySector {
    data: MemorySector,
}

impl ReadMemory for InternalRamMemorySector {
    fn read_byte(&self, position: Word) -> Byte {
        self.data.read_byte(position)
    }

    fn read_word(&self, position: Word) -> Word {
        self.data.read_word(position)
    }
}

impl WriteMemory for InternalRamMemorySector {
    fn write_byte(&mut self, position: Word, value: Byte) {
        self.data.write_byte(position, value);
    }

    fn write_word(&mut self, position: Word, value: Word) {
        self.data.write_word(position, value);
    }
}

impl Default for InternalRamMemorySector {
    fn default() -> Self {
        Self {
            data: MemorySector::with_size(0x7F),
        }
    }
}
