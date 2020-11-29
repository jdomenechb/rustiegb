use crate::memory::memory_sector::{MemorySector, ReadMemory, WriteMemory};
use crate::memory::oam_entry::OamEntry;

pub struct OamMemorySector {
    data: MemorySector,
    count: u16,
}

impl OamMemorySector {
    fn read_oam_entry(&self, position: u16) -> OamEntry {
        OamEntry::from_bytes(
            self.data.read_8(position),
            self.data.read_8(position + 1),
            self.data.read_8(position + 2),
            self.data.read_8(position + 3),
        )
    }
}

impl ReadMemory for OamMemorySector {
    fn read_8(&self, position: u16) -> u8 {
        self.data.read_8(position)
    }

    fn read_16(&self, position: u16) -> u16 {
        self.data.read_16(position)
    }
}

impl WriteMemory for OamMemorySector {
    fn write_8(&mut self, position: u16, value: u8) {
        self.data.write_8(position, value);
    }

    fn write_16(&mut self, position: u16, value: u16) {
        self.data.write_16(position, value);
    }
}

impl Iterator for OamMemorySector {
    type Item = OamEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.data.len() as u16 {
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
            data: MemorySector::new(0xA0),
            count: 0,
        }
    }
}
