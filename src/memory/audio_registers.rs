use crate::audio::sweep::Sweep;
use crate::audio::{VolumeEnvelopeDirection, WaveOutputLevel};
use crate::Byte;

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

    pub fn calculate_frequency(&self) -> f32 {
        let frequency = ((self.control as u16 & 0b111) << 8) | self.frequency as u16;
        return 131072 as f32 / (2048 - frequency) as f32;
    }

    pub fn calculate_wave_frequency(&self) -> f32 {
        let frequency = ((self.control as u16 & 0b111) << 8) | self.frequency as u16;
        return 65536 as f32 / (2048 - frequency) as f32;
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

    pub fn get_use_length(&self) -> bool {
        self.control & 0b1000000 == 0b1000000
    }

    pub fn get_length(&self) -> Byte {
        self.length
    }

    pub fn get_should_play(&self) -> bool {
        if let Some(sweep) = self.sweep {
            return sweep & 0b10000000 == 0b10000000;
        }

        true
    }

    pub fn get_sweep(&self) -> Option<Sweep> {
        if let Some(sweep) = self.sweep {
            return Some(Sweep::from(sweep));
        }

        None
    }
}
