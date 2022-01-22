use crate::cartridge::cartridge_type::CartridgeType;
use crate::cartridge::ram_size::RamSize;
use crate::cartridge::rom_size::RomSize;
use crate::Byte;

#[readonly::make]
#[derive(Debug)]
pub struct CartridgeHeader {
    pub title: String,
    pub cartridge_type: CartridgeType,
    pub rom_size: RomSize,
    pub ram_size: RamSize,
}

impl CartridgeHeader {
    pub fn new(title: String, cartridge_type: Byte, rom_size: Byte, ram_size: Byte) -> Self {
        Self {
            title,
            cartridge_type: cartridge_type.into(),
            rom_size: rom_size.into(),
            ram_size: ram_size.into(),
        }
    }

    pub fn new_from_data(data: &[Byte]) -> Self {
        let slice = &data[0x134..0x143];
        let title_chars = slice.iter().map(|b| *b as char).collect::<Vec<_>>();

        let title = title_chars.iter().collect::<String>();

        Self::new(
            title.trim_end_matches('\0').to_string(),
            data[0x147],
            data[0x148],
            data[0x149],
        )
    }
}

impl Default for CartridgeHeader {
    fn default() -> Self {
        Self::new("EMPTY TITLE".to_string(), 0, 0, 0)
    }
}
