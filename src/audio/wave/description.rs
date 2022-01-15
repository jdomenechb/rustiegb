use crate::audio::registers::{
    ChannelStopabble, ControlRegisterUpdatable, ControlUpdatable, FrequencyRegisterUpdatable,
    FrequencyUpdatable, LengthRegisterUpdatable, LengthUpdatable,
};
use crate::audio::wave::WaveOutputLevel;
use crate::memory::wave_pattern_ram::WavePatternRam;
use crate::{Byte, Word};

#[derive(Default)]
pub struct WaveDescription {
    pub set: bool,
    pub frequency: u16,
    pub output_level: WaveOutputLevel,
    pub wave: WavePatternRam,
    pub use_length: bool,
    pub length: Byte,
    pub remaining_steps: Word,
    pub should_play: bool,
    sample_clock: f32,
    pub stop: bool,
}

impl WaveDescription {
    pub fn step_256(&mut self) {
        if self.use_length && self.remaining_steps > 0 {
            self.clock_length()
        }
    }

    pub fn calculate_frequency(&self) -> f32 {
        65536.0 / (2048 - self.frequency) as f32
    }

    pub fn next_sample_clock(&mut self) -> f32 {
        let value = self.sample_clock;
        self.sample_clock += 1.0;

        value
    }

    pub fn trigger_wave_onoff_register_update(&mut self, register: Byte) {
        self.should_play = register & 0b10000000 == 0b10000000;

        if !self.should_play {
            self.stop_channel();
        }
    }

    pub fn trigger_wave_output_level_register_update(&mut self, register: Byte) {
        self.output_level = WaveOutputLevel::from(register);
    }

    pub fn trigger_wave_pattern_update(&mut self, pattern: WavePatternRam) {
        self.wave = pattern;
    }
}

impl LengthUpdatable for WaveDescription {
    fn get_maximum_length() -> Word {
        256
    }

    fn calculate_length_from_register(register: Byte) -> Byte {
        register
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

impl LengthRegisterUpdatable for WaveDescription {
    fn trigger_length_register_update(&mut self, register: Byte) {
        self.update_length_from_register(register);
    }
}

impl ControlUpdatable for WaveDescription {}

impl ControlRegisterUpdatable for WaveDescription {
    fn trigger_control_register_update(&mut self, register: Byte, next_frame_step_is_length: bool) {
        self.stop = false;

        self.set_freq_high_part_from_register(register);

        let new_use_length = Self::calculate_use_length_from_register(register);
        let old_use_length = self.use_length;

        self.set = Self::calculate_initial_from_register(register);
        self.use_length = new_use_length;

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
            }
        }

        if !self.should_play {
            self.stop_channel();
        }
    }
}

impl FrequencyUpdatable for WaveDescription {
    fn set_frequency(&mut self, frequency: Word) {
        self.frequency = frequency;
    }

    fn get_frequency(&self) -> Word {
        self.frequency
    }
}

impl FrequencyRegisterUpdatable for WaveDescription {}

impl ChannelStopabble for WaveDescription {
    fn stop_channel(&mut self) {
        self.stop = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stops_when_no_remaining_steps() {
        let mut wd = WaveDescription::default();
        wd.use_length = true;
        wd.trigger_length_register_update(255);

        assert_eq!(wd.remaining_steps, 1);
        assert_eq!(wd.stop, false);

        wd.step_256();

        assert_eq!(wd.remaining_steps, 0);
        assert_eq!(wd.stop, true);
    }
}
