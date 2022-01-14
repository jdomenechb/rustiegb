use crate::Byte;

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum PulseWavePatternDuty {
    Percent125,
    Percent25,
    Percent50,
    Percent75,
}

impl PulseWavePatternDuty {
    pub fn to_percent(&self) -> f32 {
        match self {
            Self::Percent125 => 0.125,
            Self::Percent25 => 0.25,
            Self::Percent50 => 0.50,
            Self::Percent75 => 0.75,
        }
    }
}

impl Default for PulseWavePatternDuty {
    fn default() -> Self {
        Self::Percent50
    }
}

impl From<Byte> for PulseWavePatternDuty {
    fn from(wave_duty: Byte) -> Self {
        match wave_duty {
            0b00 => Self::Percent125,
            0b01 => Self::Percent25,
            0b10 => Self::Percent50,
            0b11 => Self::Percent75,
            _ => panic!("Invalid Wave Duty"),
        }
    }
}
