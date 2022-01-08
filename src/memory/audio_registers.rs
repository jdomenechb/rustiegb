use crate::audio::volume_envelope::VolumeEnvelopeDirection;
use crate::Byte;

#[readonly::make]
pub struct AudioRegisters {
    pub control: Byte,
    pub frequency: Byte,
    pub envelope_or_wave_out_lvl: Byte,
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
            envelope_or_wave_out_lvl: envelope,
            length,
            sweep,
        }
    }

    pub fn get_volume_envelope(&self) -> Byte {
        (self.envelope_or_wave_out_lvl >> 4) & 0xF
    }

    pub fn get_volume_envelope_direction(&self) -> VolumeEnvelopeDirection {
        VolumeEnvelopeDirection::from(self.envelope_or_wave_out_lvl & 0b1000 == 0b1000)
    }

    pub fn get_volume_envelope_duration_64(&self) -> Byte {
        self.envelope_or_wave_out_lvl & 0b111
    }

    pub fn is_set(&self) -> bool {
        self.control & 0b10000000 == 0b10000000
    }

    pub fn is_length_used(&self) -> bool {
        self.control & 0b1000000 == 0b1000000
    }

    pub fn get_pulse_or_noise_length(&self) -> Byte {
        self.length & 0b111111
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
