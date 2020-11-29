pub trait ReadMemory {
    fn read_8(&self, position: u16) -> u8;
    fn read_16(&self, position: u16) -> u16;
}

pub trait WriteMemory {
    fn write_8(&mut self, position: u16, value: u8);
    fn write_16(&mut self, position: u16, value: u16);
}

pub struct MemorySector {
    data: Vec<u8>,
}

impl MemorySector {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    pub fn with_data(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl ReadMemory for MemorySector {
    fn read_8(&self, position: u16) -> u8 {
        return self.data[position as usize];
    }

    fn read_16(&self, position: u16) -> u16 {
        let position = position as usize;
        let mut result: u16 = self.data[position] as u16;
        result += (self.data[position + 1] as u16) << 8;
        return result;
    }
}

impl WriteMemory for MemorySector {
    fn write_8(&mut self, position: u16, value: u8) {
        self.data[position as usize] = value;
    }

    fn write_16(&mut self, position: u16, value: u16) {
        let position = position as usize;
        self.data[position] = value as u8;
        self.data[position + 1] = (value >> 8) as u8;
    }
}
