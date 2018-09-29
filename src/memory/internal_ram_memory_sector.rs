pub struct InternalRamMemorySector {
    data: [u8; 0x7F]
}

impl InternalRamMemorySector {
    pub fn new() -> InternalRamMemorySector {
        return InternalRamMemorySector {
            data: [0; 0x7F]
        };
    }

    /**
     * Reads a 8bit value from memory.
     */
    pub fn read_8(&self, position: u16) -> u8 {
        return self.data[position as usize];
    }

    /**
     * Reads a 16bit value from memory. First byte is lower part, second is higher.
     */
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
}