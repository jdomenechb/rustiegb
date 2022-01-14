use crate::audio::registers::{
    ChannelStopabble, ControlRegisterUpdatable, ControlUpdatable, EnvelopeRegisterUpdatable,
    EnvelopeUpdatable, LengthRegisterUpdatable, LengthUpdatable,
};
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
    pub fn step_64(&mut self) {
        self.volume_envelope.step_64();
    }

    pub fn step_256(&mut self) {
        if self.use_length && self.remaining_steps > 0 {
            self.clock_length()
        }
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

    pub fn trigger_poly_counter_register_update(&mut self, register: Byte) {
        self.poly_div_ratio = register & 0b111;
        self.poly_step = register & 0b1000 == 0b1000;
        self.poly_shift_clock_freq = register >> 4;
    }
}

impl ChannelStopabble for NoiseDescription {
    fn stop_channel(&mut self) {
        self.stop = true;
    }
}

impl LengthUpdatable for NoiseDescription {
    fn get_maximum_length() -> Word {
        64
    }

    fn calculate_length_from_register(register: Byte) -> Byte {
        register & 0b111111
    }

    fn set_length(&mut self, length: Byte) {
        self.length = length;
    }

    fn get_length(&mut self) -> Byte {
        self.length
    }

    fn set_remaining_steps(&mut self, remaining_steps: Word) {
        self.remaining_steps = remaining_steps;
    }

    fn clock_length(&mut self) {
        self.remaining_steps -= 1;

        if self.remaining_steps == 0 {
            self.stop_channel();
        }
    }
}

impl LengthRegisterUpdatable for NoiseDescription {
    fn trigger_length_register_update(&mut self, register: Byte) {
        self.update_length_from_register(register);
    }
}

impl EnvelopeUpdatable for NoiseDescription {
    fn set_envelope(&mut self, envelope: VolumeEnvelopeDescription) {
        self.volume_envelope = envelope;
    }
}

impl EnvelopeRegisterUpdatable for NoiseDescription {}

impl ControlUpdatable for NoiseDescription {}

impl ControlRegisterUpdatable for NoiseDescription {
    fn trigger_control_register_update(&mut self, register: Byte, next_frame_step_is_length: bool) {
        self.stop = false;

        let new_use_length = Self::calculate_use_length_from_register(register);
        let old_use_length = self.use_length;

        self.set = Self::calculate_initial_from_register(register);
        self.use_length = new_use_length;

        let mut steps_resetted = false;

        if self.set {
            self.sample_clock = 0.0;
            if self.remaining_steps == 0 {
                self.set_remaining_steps(Self::get_maximum_length());
                steps_resetted = true;
            }
        }

        if !next_frame_step_is_length
            && (!old_use_length || steps_resetted)
            && new_use_length
            && self.remaining_steps > 0
        {
            self.clock_length();
        }

        if self.volume_envelope.is_disabled() {
            self.stop_channel();
        }
    }
}
