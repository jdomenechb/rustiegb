use crate::Byte;

#[derive(Debug, PartialEq)]
pub enum RomSize {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ok() {
        assert_eq!(RomSize::from(0x00), RomSize::Kb32);
        assert_eq!(RomSize::from(0x01), RomSize::Kb64);
        assert_eq!(RomSize::from(0x02), RomSize::Kb128);
        assert_eq!(RomSize::from(0x03), RomSize::Kb256);
        assert_eq!(RomSize::from(0x04), RomSize::Kb512);
        assert_eq!(RomSize::from(0x05), RomSize::Mb1);
        assert_eq!(RomSize::from(0x06), RomSize::Mb2);
        assert_eq!(RomSize::from(0x07), RomSize::Mb4);
        assert_eq!(RomSize::from(0x08), RomSize::Mb8);
        assert_eq!(RomSize::from(0x52), RomSize::Mb1d1);
        assert_eq!(RomSize::from(0x53), RomSize::Mb1d2);
        assert_eq!(RomSize::from(0x54), RomSize::Mb1d5);
    }

    #[test]
    #[should_panic(expected = "Invalid ROM size")]
    fn test_from_ko() {
        let _ = RomSize::from(0xFF);
    }
}
