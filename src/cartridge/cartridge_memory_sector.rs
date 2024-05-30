use crate::Byte;

pub trait ReadCartridgeMemory {
    fn read_byte(&self, position: usize) -> Byte;
}

pub trait WriteCartridgeMemory {
    fn write_byte(&mut self, position: usize, value: Byte);
}

pub struct CartridgeMemorySector {
    data: Vec<Byte>,
}

impl CartridgeMemorySector {
    pub fn of_size(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    pub fn new_from_data(data: Vec<Byte>) -> Self {
        Self { data }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}

impl ReadCartridgeMemory for CartridgeMemorySector {
    fn read_byte(&self, position: usize) -> Byte {
        self.data[position]
    }
}

impl WriteCartridgeMemory for CartridgeMemorySector {
    fn write_byte(&mut self, position: usize, value: Byte) {
        self.data[position] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_write_out_of_bonds_panics() {
        let mut cartridge_sector = CartridgeMemorySector::of_size(4);
        cartridge_sector.write_byte(4, 0xFF);

        assert_eq!(cartridge_sector.data.len(), 0);
    }
}
