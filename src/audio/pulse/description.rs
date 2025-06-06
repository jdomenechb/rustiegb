use crate::audio::pulse::PulseWavePatternDuty;
use crate::audio::pulse::sweep::Sweep;
use crate::audio::registers::{
    ChannelStopabble, ControlRegisterUpdatable, ControlUpdatable, EnvelopeRegisterUpdatable,
    EnvelopeUpdatable, FrequencyRegisterUpdatable, FrequencyUpdatable, LengthRegisterUpdatable,
    LengthUpdatable,
};
use crate::audio::volume_envelope::VolumeEnvelopeDescription;
use crate::io::registers::IORegisters;
use crate::{Byte, Word};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Default)]
pub struct PulseDescription {
    pub set: bool,
    pub frequency: Word,
    pub wave_duty: PulseWavePatternDuty,
    pub volume_envelope: VolumeEnvelopeDescription,
    sweep: Option<Sweep>,
    pub stop: bool,
    use_length: bool,
    length: Byte,
    remaining_steps: Word,
    sample_clock: f32,
}

impl PulseDescription {
    pub fn init_sweep(&mut self) {
        self.sweep = Some(Sweep::default());
    }

    pub fn step_128(&mut self, io_registers: Arc<RwLock<IORegisters>>) {
        if let Some(mut sweep) = self.sweep {
            sweep.step_128(io_registers, self);
            self.sweep = Some(sweep);
        }
    }

    pub fn step_64(&mut self) {
        self.volume_envelope.step_64();
    }

    pub fn step_256(&mut self) {
        if self.use_length && self.remaining_steps > 0 {
            self.clock_length()
        }
    }

    pub fn calculate_frequency(&self) -> f32 {
        131072.0 / (2048.0 - self.frequency as f32)
    }

    pub fn reload_sweep(&mut self, register: Byte) {
        if let Some(mut s2) = self.sweep {
            s2.update_from_register(register, self);
            self.sweep = Some(s2);
        }
    }

    pub fn next_sample_clock(&mut self) -> f32 {
        let value = self.sample_clock;
        self.sample_clock += 1.0;

        value
    }
}

impl LengthUpdatable for PulseDescription {
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

impl LengthRegisterUpdatable for PulseDescription {
    fn trigger_length_register_update(&mut self, register: Byte) {
        self.update_length_from_register(register);

        let wave_duty = (register >> 6) & 0b11;

        self.wave_duty = wave_duty.into()
    }
}

impl ControlUpdatable for PulseDescription {}

impl ControlRegisterUpdatable for PulseDescription {
    fn trigger_control_register_update(&mut self, register: Byte, next_frame_step_is_length: bool) {
        self.stop = false;

        let new_use_length = Self::calculate_use_length_from_register(register);
        let old_use_length = self.use_length;

        self.set = Self::calculate_initial_from_register(register);
        self.use_length = new_use_length;

        self.set_freq_high_part_from_register(register);

        if let Some(mut s) = self.sweep {
            s.trigger_control_register_update(self);
            self.sweep = Some(s);
        }

        if !next_frame_step_is_length
            && !old_use_length
            && new_use_length
            && self.remaining_steps > 0
        {
            self.clock_length();
        }

        if self.set {
            self.sample_clock = 0.0;

            if self.remaining_steps == 0 {
                let mut length = Self::get_maximum_length();

                if !next_frame_step_is_length && new_use_length {
                    length -= 1;
                }

                self.set_remaining_steps(length);
                self.stop = false;
            }
        }

        if self.volume_envelope.is_disabled() {
            self.stop_channel();
        }
    }
}

impl ChannelStopabble for PulseDescription {
    fn stop_channel(&mut self) {
        self.stop = true;
    }
}

impl EnvelopeUpdatable for PulseDescription {
    fn set_envelope(&mut self, envelope: VolumeEnvelopeDescription) {
        self.volume_envelope = envelope;
    }
}

impl EnvelopeRegisterUpdatable for PulseDescription {}

impl FrequencyUpdatable for PulseDescription {
    fn set_frequency(&mut self, frequency: Word) {
        self.frequency = frequency;
    }

    fn get_frequency(&self) -> Word {
        self.frequency
    }
}

impl FrequencyRegisterUpdatable for PulseDescription {}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn test_stops_when_no_remaining_steps() {
        let mut pd = PulseDescription::default();

        pd.set_length(63);
        pd.refresh_remaining_steps();
        pd.use_length = true;

        assert_eq!(pd.remaining_steps, 1);
        assert_eq!(pd.stop, false);

        pd.step_256();

        assert_eq!(pd.remaining_steps, 0);
        assert_eq!(pd.stop, true);
    }

    #[test_case(0b01000000, 0b01)]
    #[test_case(0b10000000, 0b10)]
    fn test_correct_importation_of_wave_duty(register: Byte, expected: Byte) {
        let mut pd = PulseDescription::default();

        assert_eq!(pd.wave_duty, PulseWavePatternDuty::default());

        pd.trigger_length_register_update(register);

        assert_eq!(pd.wave_duty, PulseWavePatternDuty::from(expected));
    }
}
