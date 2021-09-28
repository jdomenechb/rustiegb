use crate::cartridge::cartridge_type::CartridgeType;
use crate::cartridge::ram_size::RamSize;
use crate::cartridge::rom_size::RomSize;
use crate::Byte;

#[readonly::make]
#[derive(Debug)]
pub struct CartridgeHeader {
    pub title: String,
    pub cartridge_type: CartridgeType,
    rom_size: RomSize,
    pub ram_size: RamSize,
}

impl CartridgeHeader {
    pub fn new_from_data(data: &Vec<Byte>) -> Self {
        let slice = &data[0x134..0x143];
        let title_chars = slice.iter().map(|b| *b as char).collect::<Vec<_>>();

        let title = title_chars.iter().collect::<String>();

        Self {
            title: title.trim_end_matches('\0').to_string(),
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
