mod cartridge_memory_sector;

use crate::cartridge::cartridge_memory_sector::{CartridgeMemorySector, ReadCartridgeMemory};
use crate::math::{two_bytes_to_word, word_to_two_bytes};
use crate::memory::memory_sector::{MemorySector, ReadMemory, WriteMemory};
use crate::{Byte, Word};
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
enum CartridgeType {
    // ROM + RAM + Battery
    Rom(bool, bool),
    // MBC1 + RAM + Battery
    Mbc1(bool, bool),
    // MBC2 + BATTERY
    Mbc2(bool),
    // MMM01 + RAM + BATTERY
    Mmm01(bool, bool),
    // MBC3 + TIMER + RAM + BATTERY
    Mbc3(bool, bool, bool),
    // MBC5 + RUMBLE + RAM + BATTERY
    Mbc5(bool, bool, bool),
    Mbc6,
    // MBC7+SENSOR+RUMBLE+RAM+BATTERY
    Mbc7,
    PocketCamera,
    BandaiTama5,
    HuC3,
    HuC1,
}

impl From<Byte> for CartridgeType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Rom(false, false),

            0x01 => Self::Mbc1(false, false),
            0x02 => Self::Mbc1(true, false),
            0x03 => Self::Mbc1(true, true),

            0x05 => Self::Mbc2(false),
            0x06 => Self::Mbc2(true),

            0x08 => Self::Rom(true, false),
            0x09 => Self::Rom(true, true),

            0x0b => Self::Mmm01(false, false),
            0x0c => Self::Mmm01(true, false),
            0x0d => Self::Mmm01(true, true),

            0x0f => Self::Mbc3(true, false, true),
            0x10 => Self::Mbc3(true, true, true),
            0x11 => Self::Mbc3(false, false, false),
            0x12 => Self::Mbc3(false, true, false),
            0x13 => Self::Mbc3(false, true, true),

            0x19 => Self::Mbc5(false, false, false),
            0x1A => Self::Mbc5(false, true, false),
            0x1B => Self::Mbc5(false, true, true),
            0x1C => Self::Mbc5(true, false, false),
            0x1D => Self::Mbc5(true, true, false),
            0x1E => Self::Mbc5(true, true, true),

            0x20 => Self::Mbc6,
            0x22 => Self::Mbc7,
            0xFC => Self::PocketCamera,
            0xFD => Self::BandaiTama5,
            0xFE => Self::HuC3,
            0xFF => Self::HuC1,

            _ => panic!("Invalid cartridge type value"),
        }
    }
}

#[derive(Debug)]
enum RomSize {
    Kb32,
    Kb64,
    Kb128,
    Kb256,
    Kb512,
    Mb1,
    Mb2,
    Mb4,
    Mb8,
    Mb1d1,
    Mb1d2,
    Mb1d5,
}

impl From<Byte> for RomSize {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Kb32,
            0x01 => Self::Kb64,
            0x02 => Self::Kb128,
            0x03 => Self::Kb256,
            0x04 => Self::Kb512,
            0x05 => Self::Mb1,
            0x06 => Self::Mb2,
            0x07 => Self::Mb4,
            0x08 => Self::Mb8,
            0x52 => Self::Mb1d1,
            0x53 => Self::Mb1d2,
            0x54 => Self::Mb1d5,
            _ => panic!("Invalid ROM size"),
        }
    }
}

#[derive(Debug)]
enum RamSize {
    None,
    Kb2,
    Kb8,
    Kb32,
    Kb128,
    Kb64,
}

impl From<Byte> for RamSize {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::None,
            0x01 => Self::Kb2,
            0x02 => Self::Kb8,
            0x03 => Self::Kb32,
            0x04 => Self::Kb128,
            0x05 => Self::Kb64,
            _ => panic!("Invalid RAM size"),
        }
    }
}

impl From<&RamSize> for MemorySector {
    fn from(value: &RamSize) -> Self {
        MemorySector::with_size(match value {
            RamSize::None => 0,
            RamSize::Kb2 => 2 * 1024,
            RamSize::Kb8 => 8 * 1024,
            RamSize::Kb32 => 32 * 1024,
            RamSize::Kb128 => 128 * 1024,
            RamSize::Kb64 => 64 * 1024,
            _ => panic!("Invalid RAM size"),
        })
    }
}

#[readonly::make]
#[derive(Debug)]
pub struct CartridgeHeader {
    pub title: String,
    cartridge_type: CartridgeType,
    rom_size: RomSize,
    ram_size: RamSize,
}

impl CartridgeHeader {
    fn new_from_data(data: &Vec<Byte>) -> Self {
        let mut title = String::from_utf8((&data[0x134..0x144]).to_vec());

        if title.is_err() {
            title = String::from_utf8((&data[0x134..0x144]).to_vec());

            if title.is_err() {
                title = Ok("".to_string());
            }
        }

        Self {
            title: title.unwrap().trim_end_matches("\0").to_string(),
            cartridge_type: data[0x147].into(),
            rom_size: data[0x148].into(),
            ram_size: data[0x149].into(),
        }
    }
}

impl Default for CartridgeHeader {
    fn default() -> Self {
        Self {
            title: "EMPTY TITLE".to_string(),
            cartridge_type: CartridgeType::Rom(false, false),
            rom_size: RomSize::Kb32,
            ram_size: RamSize::None,
        }
    }
}

#[readonly::make]
pub struct Cartridge {
    pub data: CartridgeMemorySector,
    pub header: CartridgeHeader,
    selected_rom_bank: u8,
    selected_ram_bank: u8,
    ram: MemorySector,
    ram_enabled: bool,
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
        let ram = (&header.ram_size).into();

        Self {
            data: CartridgeMemorySector::with_data(data),
            header,
            selected_rom_bank: 1,
            selected_ram_bank: 0,
            ram,
            ram_enabled: false,
            ram_banking_mode: false,
        }
    }
}

impl Default for Cartridge {
    fn default() -> Self {
        Self {
            data: CartridgeMemorySector::with_size(0),
            header: CartridgeHeader::default(),
            selected_rom_bank: 1,
            selected_ram_bank: 0,
            ram: MemorySector::with_size(0),
            ram_enabled: false,
            ram_banking_mode: false,
        }
    }
}

impl ReadMemory for Cartridge {
    fn read_byte(&self, position: Word) -> Byte {
        match self.header.cartridge_type {
            CartridgeType::Rom(false, false) => self.data.read_byte(position as usize),
            CartridgeType::Mbc1(_, _) => {
                if position < 0x4000 {
                    return self.data.read_byte(position as usize);
                }

                if position >= 0x4000 && position < 0x8000 {
                    return self.data.read_byte(
                        position as usize - 0x4000 + 0x4000 * self.selected_rom_bank as usize,
                    );
                }

                if position >= 0xA000 && position < 0xBFFF {
                    if !self.ram_enabled {
                        return 0xFF;
                    }

                    return self.ram.read_byte(
                        position as u16 - 0xA000 + 0xA000 * self.selected_ram_bank as u16,
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

    fn read_word(&self, position: Word) -> Word {
        let value1 = self.read_byte(position);
        let value2 = self.read_byte(position + 1);

        two_bytes_to_word(value2, value1)
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
            }

            CartridgeType::Mbc1(_, _) => {
                if position < 0x2000 {
                    self.ram_enabled = value & 0xA == 0xA;
                    return;
                }

                if position >= 0x2000 && position < 0x4000 {
                    self.selected_rom_bank = if value != 0 { value & 0b11111 } else { 1 };
                    return;
                }

                if position >= 0x4000 && position < 0x6000 {
                    let new_value = value & 0b11;

                    if !self.ram_banking_mode {
                        self.selected_rom_bank =
                            new_value >> 5 | (self.selected_rom_bank & 0b11111);
                    } else {
                        self.selected_ram_bank = new_value;
                    }

                    return;
                }

                if position >= 0x6000 && position < 0x8000 {
                    self.ram_banking_mode = value != 0;
                    return;
                }

                if position >= 0xA000 && position < 0xC000 {
                    if self.ram_enabled {
                        self.ram.write_byte(
                            position - 0xA000 + 0x2000 * self.selected_ram_bank as u16,
                            value,
                        );
                    }
                    return;
                }

                panic!(
                    "Writing to address {:X} into ROM space for cartridge type {:?} is not implemented",
                    position,
                    self.header.cartridge_type
                );
            }

            _ => {
                panic!(
                    "Writing to address {:X} into ROM space for cartridge type {:?} is not implemented",
                    position,
                    self.header.cartridge_type
                );
            }
        }
    }

    fn write_word(&mut self, position: u16, value: u16) {
        let bytes = word_to_two_bytes(value);

        self.write_byte(position, bytes.1);
        self.write_byte(position + 1, bytes.0);
    }
}
