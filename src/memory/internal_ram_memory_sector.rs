use crate::memory::memory_sector::{MemorySector, ReadMemory, WriteMemory};

pub struct InternalRamMemorySector {
    data: MemorySector,
}

impl ReadMemory for InternalRamMemorySector {
    fn read_byte(&self, position: u16) -> u8 {
        self.data.read_byte(position)
    }

    fn read_word(&self, position: u16) -> u16 {
        self.data.read_word(position)
    }
}

impl WriteMemory for InternalRamMemorySector {
    fn write_byte(&mut self, position: u16, value: u8) {
        self.data.write_byte(position, value);
    }

    fn write_word(&mut self, position: u16, value: u16) {
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
