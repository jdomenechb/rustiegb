use crate::memory::memory_sector::{MemorySector, ReadMemory, WriteMemory};
use crate::{Byte, Word};

pub struct WavePatternRam {
    pub data: MemorySector,
}

impl ReadMemory for WavePatternRam {
    fn read_byte(&self, position: Word) -> Byte {
        self.data.read_byte(position)
    }
}

impl WriteMemory for WavePatternRam {
    fn write_byte(&mut self, position: Word, value: Byte) {
        self.data.write_byte(position, value);
    }
}

impl Default for WavePatternRam {
    fn default() -> Self {
        Self {
            data: MemorySector::with_size(0x10),
        }
    }
}
