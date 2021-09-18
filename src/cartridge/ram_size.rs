use crate::Byte;

#[derive(Debug, PartialEq)]
pub enum RamSize {
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

impl RamSize {
    pub fn in_bytes(&self) -> usize {
        let mul = match self {
            Self::None => 0,
            Self::Kb2 => 2,
            Self::Kb8 => 8,
            Self::Kb32 => 32,
            Self::Kb128 => 128,
            Self::Kb64 => 64,
        };

        mul * 1024
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ok() {
        assert_eq!(RamSize::from(0x00), RamSize::None);
        assert_eq!(RamSize::from(0x01), RamSize::Kb2);
        assert_eq!(RamSize::from(0x02), RamSize::Kb8);
        assert_eq!(RamSize::from(0x03), RamSize::Kb32);
        assert_eq!(RamSize::from(0x04), RamSize::Kb128);
        assert_eq!(RamSize::from(0x05), RamSize::Kb64);
    }

    #[test]
    #[should_panic(expected = "Invalid RAM size")]
    fn test_from_ko() {
        RamSize::from(0xFF);
    }

    #[test]
    fn test_in_bytes_ok() {
        assert_eq!(RamSize::None.in_bytes(), 0);
        assert_eq!(RamSize::Kb2.in_bytes(), 2 * 1024);
        assert_eq!(RamSize::Kb8.in_bytes(), 8 * 1024);
        assert_eq!(RamSize::Kb32.in_bytes(), 32 * 1024);
        assert_eq!(RamSize::Kb128.in_bytes(), 128 * 1024);
        assert_eq!(RamSize::Kb64.in_bytes(), 64 * 1024);
    }
}
