use crate::audio::sweep::Sweep;
use crate::audio::{VolumeEnvelopeDirection, WaveOutputLevel};
use crate::{Byte, Word};

#[derive(Clone)]
pub enum PulseWavePatternDuty {
    Percent125,
    Percent25,
    Percent50,
    Percent75,
}

impl PulseWavePatternDuty {
    pub fn calculate_amplitude(&self, position: Byte) -> f32 {
        match self {
            Self::Percent125 => {
                if position == 7 {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Percent25 => {
                if position > 5 {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Percent50 => {
                if position > 3 {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Percent75 => {
                if position < 6 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

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

#[readonly::make]
pub struct AudioRegisters {
    pub control: Byte,
    pub frequency: Byte,
    pub volume: Byte,
    pub length: Byte,
    pub sweep: Option<Byte>,
}

impl AudioRegisters {
    pub fn new(
        control: Byte,
        frequency: Byte,
        volume: Byte,
        length: Byte,
        sweep: Option<Byte>,
    ) -> Self {
        Self {
            control,
            frequency,
            volume,
            length,
            sweep,
        }
    }

    pub fn get_frequency(&self) -> Word {
        ((self.control as u16 & 0b111) << 8) | self.frequency as u16
    }

    pub fn calculate_wave_duty(&self) -> PulseWavePatternDuty {
        let wave_duty = (self.length >> 6) & 0b11;

        wave_duty.into()
    }

    pub fn get_volume_envelope(&self) -> Byte {
        (self.volume >> 4) & 0xF
    }

    pub fn get_volume_envelope_direction(&self) -> VolumeEnvelopeDirection {
        VolumeEnvelopeDirection::from(self.volume & 0b1000 == 0b1000)
    }

    pub fn get_volume_envelope_duration_64(&self) -> Byte {
        self.volume & 0b111
    }

    pub fn get_wave_output_level(&self) -> WaveOutputLevel {
        match self.volume & 0b1100000 {
            0b0000000 => WaveOutputLevel::Mute,
            0b0100000 => WaveOutputLevel::Vol100Percent,
            0b1000000 => WaveOutputLevel::Vol50Percent,
            0b1100000 => WaveOutputLevel::Vol25Percent,
            _ => panic!("Invalid Wave Output Level"),
        }
    }

    pub fn is_set(&self) -> bool {
        self.control & 0b10000000 == 0b10000000
    }

    pub fn is_length_used(&self) -> bool {
        self.control & 0b1000000 == 0b1000000
    }

    pub fn get_wave_length(&self) -> Byte {
        self.length
    }

    pub fn get_pulse_or_noise_length(&self) -> Byte {
        self.length & 0b111111
    }

    pub fn get_wave_should_play(&self) -> bool {
        if let Some(sweep) = self.sweep {
            return sweep & 0b10000000 == 0b10000000;
        }

        true
    }

    pub fn get_sweep(&self) -> Option<Sweep> {
        if let Some(sweep) = self.sweep {
            return Some(Sweep::new(sweep, self.get_frequency()));
        }

        None
    }

    pub fn get_poly_shift_clock_freq(&self) -> Byte {
        (self.frequency >> 4) & 0xF
    }

    pub fn get_poly_step(&self) -> bool {
        (self.frequency >> 3) & 0b1 == 1
    }

    pub fn get_poly_div_ratio(&self) -> Byte {
        self.frequency & 0b111
    }
}
