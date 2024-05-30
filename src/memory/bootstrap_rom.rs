use crate::memory::memory_sector::{MemorySector, ReadMemory};
use crate::{Byte, Word};
use std::fs::File;
use std::io::Read;

pub struct BootstrapRom {
    pub data: MemorySector,
}

impl BootstrapRom {
    pub fn new_from_path(path: &str) -> Self {
        let mut bootstrap_data: Vec<Byte> = Vec::with_capacity(0x256);

        let mut bootstrap_rom_file = File::open(path).expect("Bootstrap ROM could not be open");
        bootstrap_rom_file
            .read_to_end(&mut bootstrap_data)
            .expect("Error on reading Bootstrap ROM contents");

        Self {
            data: MemorySector::with_data(bootstrap_data[0x0..bootstrap_data.len()].to_vec()),
        }
    }
}

impl ReadMemory for BootstrapRom {
    fn read_byte(&self, position: Word) -> Byte {
        self.data.read_byte(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::fixture::FileWriteBin;
    use assert_fs::NamedTempFile;

    #[test]
    fn it_reads_from_path() {
        let tmp_file = NamedTempFile::new("foo.rs").unwrap();
        tmp_file.write_binary(&[0x09_u8, 0x08_u8, 0x07_u8]).unwrap();

        let bootstrap_rom = BootstrapRom::new_from_path(tmp_file.to_str().unwrap());

        assert_eq!(bootstrap_rom.read_byte(0x0), 0x09);
        assert_eq!(bootstrap_rom.read_byte(0x1), 0x08);
        assert_eq!(bootstrap_rom.read_byte(0x2), 0x07);

        tmp_file.close().unwrap();
    }

    #[test]
    #[should_panic(expected = "index out of bounds: the len is 1 but the index is 1")]
    fn it_cannot_read_out_of_bounds() {
        let tmp_file = NamedTempFile::new("foo.rs").unwrap();
        tmp_file.write_binary(&[0x09_u8]).unwrap();

        let bootstrap_rom = BootstrapRom::new_from_path(tmp_file.to_str().unwrap());

        bootstrap_rom.read_byte(1);

        tmp_file.close().unwrap();
    }

    #[test]
    #[should_panic(expected = "Bootstrap ROM could not be open")]
    fn it_cannot_open_file_as_it_does_not_exist() {
        BootstrapRom::new_from_path("file_that_does_not_exist.bin");
    }
}
