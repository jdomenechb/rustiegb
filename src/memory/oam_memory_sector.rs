use crate::memory::oam_entry::OamEntry;

pub struct OamMemorySector {
    data: [u8; 0xA0],
    count: u16,
}

impl OamMemorySector {
    pub fn new() -> OamMemorySector {
        return OamMemorySector {
            data: [0; 0xA0],
            count: 0,
        };
    }

    pub fn read_8(&self, position: u16) -> u8 {
        return self.data[position as usize];
    }

    pub fn read_16(&self, position: u16) -> u16{
        let position = position as usize;
        let mut result: u16 = self.data[position] as u16;
        result += (self.data[position + 1] as u16) << 8;
        return result;
    }

    pub fn write_8(&mut self, position: u16, value: u8) {
        self.data[position as usize] = value;
    }

    pub fn write_16(&mut self, position: u16, value: u16) {
        let position = position as usize;
        self.data[position] = value as u8;
        self.data[position + 1] = (value >> 8) as u8;
    }

    fn read_oam_entry(&self, position: u16) -> OamEntry {
        let position = position as usize;

        OamEntry::from_bytes(
            self.data[position],
            self.data[position + 1],
            self.data[position + 2],
            self.data[position + 3]
        )
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