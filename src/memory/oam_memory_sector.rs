use crate::memory::memory_sector::{MemorySector, ReadMemory, WriteMemory};
use crate::memory::oam_entry::OamEntry;
use crate::{Byte, Word};

const OAM_MEMORY_SECTOR_SIZE: u16 = 0xA0;

pub struct OamMemorySector {
    data: MemorySector,
    count: u16,
}

impl OamMemorySector {
    fn read_oam_entry(&self, position: Word) -> OamEntry {
        OamEntry::with_bytes(
            self.data.read_byte(position),
            self.data.read_byte(position + 1),
            self.data.read_byte(position + 2),
            self.data.read_byte(position + 3),
        )
    }
}

impl ReadMemory for OamMemorySector {
    fn read_byte(&self, position: Word) -> Byte {
        self.data.read_byte(position)
    }
}

impl WriteMemory for OamMemorySector {
    fn write_byte(&mut self, position: Word, value: Byte) {
        self.data.write_byte(position, value);
    }
}

impl Iterator for OamMemorySector {
    type Item = OamEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= OAM_MEMORY_SECTOR_SIZE {
            self.count = 0;
            return None;
        }

        let result = Some(self.read_oam_entry(self.count));
        self.count += 4;

        result
    }
}

impl Default for OamMemorySector {
    fn default() -> Self {
        Self {
            data: MemorySector::with_size(OAM_MEMORY_SECTOR_SIZE as usize),
            count: 0,
        }
    }
}
