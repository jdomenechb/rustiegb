use crate::audio::volume_envelope::VolumeEnvelopeDescription;
use crate::{Byte, Word};

#[derive(Default)]
pub struct NoiseDescription {
    pub set: bool,
    pub volume_envelope: VolumeEnvelopeDescription,
    poly_shift_clock_freq: Byte,
    poly_step: bool,
    poly_div_ratio: Byte,
    pub stop: bool,
    use_length: bool,
    length: Byte,
    remaining_steps: Word,
    sample_clock: f32,
    pub lfsr: Word,
}

impl NoiseDescription {
    pub fn new(
        set: bool,
        volume_envelope: VolumeEnvelopeDescription,
        poly_shift_clock_freq: Byte,
        poly_step: bool,
        poly_div_ratio: Byte,
        use_length: bool,
        length: Byte,
    ) -> Self {
        let mut value = Self {
            set,
            volume_envelope,
            poly_shift_clock_freq,
            poly_step,
            poly_div_ratio,
            stop: false,
            use_length,
            length,
            remaining_steps: 0,
            sample_clock: 0.0,
            lfsr: 0,
        };

        value.reload_length(length);

        value
    }

    pub fn exchange(&mut self, other: &Self) {
        self.set = other.set;
        self.volume_envelope = VolumeEnvelopeDescription::new(
            other.volume_envelope.initial_volume,
            other.volume_envelope.direction,
            other.volume_envelope.period,
        );
        self.poly_shift_clock_freq = other.poly_shift_clock_freq;
        self.poly_step = other.poly_step;
        self.poly_div_ratio = other.poly_div_ratio;
        self.stop = other.stop;
        self.use_length = other.use_length;
        self.length = other.length;
        self.remaining_steps = other.remaining_steps;
        self.sample_clock = 0.0;
    }

    pub fn step_64(&mut self) {
        self.volume_envelope.step_64();
    }

    pub fn step_256(&mut self) {
        if self.use_length && self.remaining_steps > 0 {
            self.remaining_steps -= 1;

            if self.remaining_steps == 0 {
                self.stop = true;
            }
        }
    }

    pub fn reload_length(&mut self, length: Byte) {
        self.length = length;
        self.remaining_steps = 256 - length as Word;
    }

    pub fn next_sample_clock(&mut self) -> f32 {
        let value = self.sample_clock;
        self.sample_clock += 1.0;

        value
    }

    pub fn calculate_frequency(&self) -> f32 {
        let freq = 524288_f32;

        let divisor = (if self.poly_div_ratio > 0 {
            self.poly_div_ratio << 4
        } else {
            8
        }) << self.poly_shift_clock_freq;

        freq / divisor as f32
    }

    pub fn update_lfsr(&mut self) {
        let xor_result = (self.lfsr & 0b01) ^ ((self.lfsr & 0b10) >> 1);
        self.lfsr = (self.lfsr >> 1) | (xor_result << 14);

        if self.poly_step {
            self.lfsr &= !(1 << 6);
            self.lfsr |= xor_result << 6
        }
    }
}
