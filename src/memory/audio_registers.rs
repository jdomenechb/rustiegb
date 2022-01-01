use crate::audio::sweep::Sweep;
use crate::audio::{VolumeEnvelopeDirection, WaveOutputLevel};
use crate::{Byte, Word};

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

    pub fn calculate_wave_duty_percent(&self) -> f32 {
        let wave_duty = (self.length >> 6) & 0b11;

        match wave_duty {
            0b00 => 0.125,
            0b01 => 0.25,
            0b10 => 0.50,
            0b11 => 0.75,
            _ => panic!("Invalid Wave Duty"),
        }
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

    pub fn get_pulse_length(&self) -> Byte {
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
}
