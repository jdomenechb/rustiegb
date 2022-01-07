use crate::audio::pulse::sweep::Sweep;
use crate::audio::volume_envelope::VolumeEnvelopeDirection;
use crate::audio::wave::WaveOutputLevel;
use crate::{Byte, Word};

#[readonly::make]
pub struct AudioRegisters {
    pub control: Byte,
    pub frequency: Byte,
    pub envelope: Byte,
    pub length: Byte,
    pub sweep: Option<Byte>,
}

impl AudioRegisters {
    pub fn new(
        control: Byte,
        frequency: Byte,
        envelope: Byte,
        length: Byte,
        sweep: Option<Byte>,
    ) -> Self {
        Self {
            control,
            frequency,
            envelope,
            length,
            sweep,
        }
    }

    pub fn get_frequency(&self) -> Word {
        ((self.control as u16 & 0b111) << 8) | self.frequency as u16
    }

    pub fn get_volume_envelope(&self) -> Byte {
        (self.envelope >> 4) & 0xF
    }

    pub fn get_volume_envelope_direction(&self) -> VolumeEnvelopeDirection {
        VolumeEnvelopeDirection::from(self.envelope & 0b1000 == 0b1000)
    }

    pub fn get_volume_envelope_duration_64(&self) -> Byte {
        self.envelope & 0b111
    }

    pub fn get_wave_output_level(&self) -> WaveOutputLevel {
        match self.envelope & 0b1100000 {
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
