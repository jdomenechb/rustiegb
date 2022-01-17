use std::fs::File;
use std::io::Read;

use cartridge_header::CartridgeHeader;
use cartridge_type::CartridgeType;

use crate::cartridge::cartridge_memory_sector::{
    CartridgeMemorySector, ReadCartridgeMemory, WriteCartridgeMemory,
};
use crate::memory::memory_sector::{ReadMemory, WriteMemory};
use crate::{Byte, Word};

mod cartridge_header;
mod cartridge_memory_sector;
mod cartridge_type;
mod ram_size;
mod rom_size;

#[readonly::make]
pub struct Cartridge {
    pub data: CartridgeMemorySector,
    pub header: CartridgeHeader,
    selected_rom_bank: u16,
    ram_enabled: bool,
    selected_ram_bank: u8,
    ram: CartridgeMemorySector,
    ram_banking_mode: bool,
}

impl Cartridge {
    pub fn new_from_path(rom_path: &str) -> Self {
        let mut data: Vec<Byte> = Vec::new();
        let mut rom_file = File::open(rom_path).expect("File not found");
        rom_file
            .read_to_end(&mut data)
            .expect("Error on reading ROM contents");

        let header = CartridgeHeader::new_from_data(&data);

        let ram_size_in_bytes = header.ram_size.in_bytes();

        Self {
            data: CartridgeMemorySector::from_data(data),
            header,
            selected_rom_bank: 1,
            ram_enabled: false,
            selected_ram_bank: 0,
            ram: CartridgeMemorySector::of_size(ram_size_in_bytes),
            ram_banking_mode: false,
        }
    }

    pub fn print_header(&self) {
        println!("CARTRIDGE HEADER");
        println!("{:?}", self.header);
    }
}

impl Default for Cartridge {
    fn default() -> Self {
        Self {
            data: CartridgeMemorySector::of_size(0),
            header: CartridgeHeader::default(),
            selected_rom_bank: 1,
            ram_enabled: false,
            selected_ram_bank: 1,
            ram: CartridgeMemorySector::of_size(0),
            ram_banking_mode: false,
        }
    }
}

impl ReadMemory for Cartridge {
    fn read_byte(&self, position: Word) -> Byte {
        if position < 0x4000 {
            return self.data.read_byte(position as usize);
        }

        if (0x4000..0x8000).contains(&position) {
            return self
                .data
                .read_byte(position as usize - 0x4000 + 0x4000 * self.selected_rom_bank as usize);
        }

        match self.header.cartridge_type {
            CartridgeType::Rom(false, false) => self.data.read_byte(position as usize),
            CartridgeType::Mbc1(_, _) => {
                if (0xA000..0xC000).contains(&position) {
                    if !self.ram_enabled {
                        return 0xFF;
                    }

                    return self.ram.read_byte(
                        position as usize - 0xA000 + 0x2000 * self.selected_ram_bank as usize,
                    );
                }

                panic!(
                    "Reading address {:X} from ROM space for cartridge type {:?} is not implemented",
                    position,
                    self.header.cartridge_type
                );
            }
            CartridgeType::Mbc3(_, _, _) => {
                if (0xA000..0xC000).contains(&position) {
                    if !self.ram_enabled {
                        return 0xFF;
                    }

                    return self.ram.read_byte(
                        position as usize - 0xA000 + 0x2000 * self.selected_ram_bank as usize,
                    );
                }

                panic!(
                    "Reading address {:X} from ROM space for cartridge type {:?} is not implemented",
                    position,
                    self.header.cartridge_type
                );
            }
            CartridgeType::Mbc5(_, _, _) => {
                if (0xA000..0xC000).contains(&position) {
                    if !self.ram_enabled {
                        return 0xFF;
                    }

                    return self.ram.read_byte(
                        position as usize - 0xA000 + 0x2000 * self.selected_ram_bank as usize,
                    );
                }

                panic!(
                    "Reading address {:X} from ROM space for cartridge type {:?} is not implemented",
                    position,
                    self.header.cartridge_type
                );
            }
            _ => {
                panic!(
                    "Reading address {:X} from ROM space for cartridge type {:?} is not implemented",
                    position,
                    self.header.cartridge_type
                );
            }
        }
    }
}

impl WriteMemory for Cartridge {
    fn write_byte(&mut self, position: u16, value: u8) {
        match self.header.cartridge_type {
            CartridgeType::Rom(false, false) => {
                println!(
                    "Attempt to write at Memory {:X}. ROM is not writable!!!",
                    position
                );

                return;
            }

            CartridgeType::Mbc1(ram, _) => {
                if self.determine_ram_enable(position, value, ram) {
                    return;
                }

                // Select ROM Bank Number
                if (0x2000..0x4000).contains(&position) {
                    let new_value = value as u16 & 0b11111;

                    self.selected_rom_bank = if new_value % 0x20 == 0 {
                        new_value + 1
                    } else {
                        new_value
                    };

                    return;
                }

                if (0x4000..0x6000).contains(&position) {
                    let new_value = value & 0b11;

                    if !self.ram_banking_mode {
                        self.selected_rom_bank =
                            (new_value as Word) << 5 | (self.selected_rom_bank & 0b11111);
                    } else {
                        self.selected_ram_bank = new_value;
                    }

                    return;
                }

                if (0x6000..0x8000).contains(&position) {
                    self.ram_banking_mode = value & 0b1 == 0b1;
                    return;
                }

                if (0xA000..0xC000).contains(&position) {
                    if self.ram_enabled {
                        self.ram.write_byte(
                            position as usize - 0xA000 + 0x2000 * self.selected_ram_bank as usize,
                            value,
                        );
                    }
                    return;
                }
            }

            CartridgeType::Mbc3(timer, ram, _) => {
                if self.determine_ram_enable(position, value, ram) {
                    return;
                }

                // Select ROM Bank Number
                if (0x2000..0x4000).contains(&position) {
                    self.selected_rom_bank = if value != 0 {
                        value as u16 & 0b1111111
                    } else {
                        1
                    };

                    return;
                }

                if (0x4000..0x6000).contains(&position) {
                    if value <= 0x7 {
                        self.selected_ram_bank = value;
                        return;
                    }

                    panic!("Writing value {:X} to address {:X} into ROM space for cartridge type {:?} is not implemented", value, position, self.header.cartridge_type);
                }

                if (0x6000..0x8000).contains(&position) {
                    if !timer {
                        return;
                    }

                    panic!("Writing value {:X} to address {:X} into ROM space for cartridge type {:?} is not implemented", value, position, self.header.cartridge_type);
                }

                if (0xA000..0xC000).contains(&position) {
                    if self.ram_enabled {
                        self.ram.write_byte(
                            position as usize - 0xA000 + 0x2000 * self.selected_ram_bank as usize,
                            value,
                        );
                    }
                    return;
                }
            }

            CartridgeType::Mbc5(_, ram, _) => {
                if self.determine_ram_enable(position, value, ram) {
                    return;
                }

                // Select ROM Bank Number - Low
                if (0x2000..0x3000).contains(&position) {
                    self.selected_rom_bank = value as Word & 0xFF;

                    return;
                }

                // Select ROM Bank Number - High
                if (0x3000..0x4000).contains(&position) {
                    self.selected_rom_bank |= ((value & 0x1) as Word) << 8;

                    return;
                }

                // Select RAM Bank Number
                if (0x4000..0x6000).contains(&position) {
                    self.selected_ram_bank = value & 0xF;
                    return;
                }

                if (0x6000..0xA000).contains(&position) {
                    // Ignore
                    return;
                }

                if (0xA000..0xC000).contains(&position) {
                    if self.ram_enabled {
                        self.ram.write_byte(
                            position as usize - 0xA000 + 0x2000 * self.selected_ram_bank as usize,
                            value,
                        );
                    }
                    return;
                }

                panic!("Writing value {:X} to address {:X} into ROM space for cartridge type {:?} is not implemented", value, position, self.header.cartridge_type);
            }
            _ => {}
        }

        panic!("Writing value {:X} to address {:X} into ROM space for cartridge type {:?} is not implemented", value, position, self.header.cartridge_type);
    }
}

impl Cartridge {
    fn determine_ram_enable(&mut self, position: u16, value: u8, ram: bool) -> bool {
        if position < 0x2000 {
            self.ram_enabled = ram && (value & 0x0A == 0x0A);
            return true;
        }

        false
    }
}
