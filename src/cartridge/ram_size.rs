use crate::Byte;

#[derive(Debug)]
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
    pub fn size(&self) -> usize {
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
