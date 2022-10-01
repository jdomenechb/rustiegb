use crate::Byte;

#[derive(PartialEq, Eq, Debug)]
pub enum CartridgeType {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ok() {
        assert_eq!(CartridgeType::from(0x00), CartridgeType::Rom(false, false));
        assert_eq!(CartridgeType::from(0x01), CartridgeType::Mbc1(false, false));
        assert_eq!(CartridgeType::from(0x02), CartridgeType::Mbc1(true, false));
        assert_eq!(CartridgeType::from(0x03), CartridgeType::Mbc1(true, true));
        assert_eq!(CartridgeType::from(0x05), CartridgeType::Mbc2(false));
        assert_eq!(CartridgeType::from(0x06), CartridgeType::Mbc2(true));
        assert_eq!(CartridgeType::from(0x08), CartridgeType::Rom(true, false));
        assert_eq!(CartridgeType::from(0x09), CartridgeType::Rom(true, true));
        assert_eq!(
            CartridgeType::from(0x0b),
            CartridgeType::Mmm01(false, false)
        );
        assert_eq!(CartridgeType::from(0x0c), CartridgeType::Mmm01(true, false));
        assert_eq!(CartridgeType::from(0x0d), CartridgeType::Mmm01(true, true));
        assert_eq!(
            CartridgeType::from(0x0f),
            CartridgeType::Mbc3(true, false, true)
        );
        assert_eq!(
            CartridgeType::from(0x10),
            CartridgeType::Mbc3(true, true, true)
        );
        assert_eq!(
            CartridgeType::from(0x11),
            CartridgeType::Mbc3(false, false, false)
        );
        assert_eq!(
            CartridgeType::from(0x12),
            CartridgeType::Mbc3(false, true, false)
        );
        assert_eq!(
            CartridgeType::from(0x13),
            CartridgeType::Mbc3(false, true, true)
        );
        assert_eq!(
            CartridgeType::from(0x19),
            CartridgeType::Mbc5(false, false, false)
        );
        assert_eq!(
            CartridgeType::from(0x1A),
            CartridgeType::Mbc5(false, true, false)
        );
        assert_eq!(
            CartridgeType::from(0x1B),
            CartridgeType::Mbc5(false, true, true)
        );
        assert_eq!(
            CartridgeType::from(0x1C),
            CartridgeType::Mbc5(true, false, false)
        );
        assert_eq!(
            CartridgeType::from(0x1D),
            CartridgeType::Mbc5(true, true, false)
        );
        assert_eq!(
            CartridgeType::from(0x1E),
            CartridgeType::Mbc5(true, true, true)
        );
        assert_eq!(CartridgeType::from(0x20), CartridgeType::Mbc6);
        assert_eq!(CartridgeType::from(0x22), CartridgeType::Mbc7);
        assert_eq!(CartridgeType::from(0xFC), CartridgeType::PocketCamera);
        assert_eq!(CartridgeType::from(0xFD), CartridgeType::BandaiTama5);
        assert_eq!(CartridgeType::from(0xFE), CartridgeType::HuC3);
        assert_eq!(CartridgeType::from(0xFF), CartridgeType::HuC1);
    }

    #[test]
    #[should_panic(expected = "Invalid cartridge type value")]
    fn test_from_ko() {
        let _ = CartridgeType::from(0x50);
    }
}
