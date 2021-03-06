use crate::memory::memory_sector::{MemorySector, ReadMemory, WriteMemory};
use crate::memory::oam_entry::OamEntry;
use crate::{Byte, Word};

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

    fn read_word(&self, position: Word) -> Word {
        self.data.read_word(position)
    }
}

impl WriteMemory for OamMemorySector {
    fn write_byte(&mut self, position: Word, value: Byte) {
        self.data.write_byte(position, value);
    }

    fn write_word(&mut self, position: Word, value: Word) {
        self.data.write_word(position, value);
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
            data: MemorySector::with_size(0xA0),
            count: 0,
        }
    }
}
