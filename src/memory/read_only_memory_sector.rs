pub struct ReadOnlyMemorySector {
    pub data: Vec<u8>
}

impl ReadOnlyMemorySector {
    pub fn new(data: Vec<u8>) -> ReadOnlyMemorySector {
        return ReadOnlyMemorySector {
            data: data
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
}