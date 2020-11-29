use crate::memory::memory_sector::{MemorySector, ReadMemory, WriteMemory};

pub struct VideoRam8kMemorySector {
    data: MemorySector,
}

impl ReadMemory for VideoRam8kMemorySector {
    fn read_8(&self, position: u16) -> u8 {
        self.data.read_8(position)
    }

    fn read_16(&self, position: u16) -> u16 {
        self.data.read_16(position)
    }
}

impl WriteMemory for VideoRam8kMemorySector {
    fn write_8(&mut self, position: u16, value: u8) {
        self.data.write_8(position, value);
    }

    fn write_16(&mut self, position: u16, value: u16) {
        self.data.write_16(position, value);
    }
}

impl Default for VideoRam8kMemorySector {
    fn default() -> Self {
        Self {
            data: MemorySector::new(0x2000),
        }
    }
}
