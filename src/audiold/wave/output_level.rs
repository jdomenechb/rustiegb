use crate::Byte;

#[derive(Eq, PartialEq, Clone, Copy, Default)]
pub enum WaveOutputLevel {
    Mute,
    #[default]
    Vol100Percent,
    Vol50Percent,
    Vol25Percent,
}

impl From<WaveOutputLevel> for f32 {
    fn from(wol: WaveOutputLevel) -> Self {
        match wol {
            WaveOutputLevel::Mute => 0.0,
            WaveOutputLevel::Vol25Percent => 0.25,
            WaveOutputLevel::Vol50Percent => 0.5,
            WaveOutputLevel::Vol100Percent => 1.0,
        }
    }
}

impl From<Byte> for WaveOutputLevel {
    fn from(register: Byte) -> Self {
        match register & 0b1100000 {
            0b0000000 => WaveOutputLevel::Mute,
            0b0100000 => WaveOutputLevel::Vol100Percent,
            0b1000000 => WaveOutputLevel::Vol50Percent,
            0b1100000 => WaveOutputLevel::Vol25Percent,
            _ => panic!("Invalid Wave Output Level"),
        }
    }
}
