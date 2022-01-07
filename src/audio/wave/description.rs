use crate::audio::wave::WaveOutputLevel;
use crate::memory::memory_sector::MemorySector;
use crate::memory::wave_pattern_ram::WavePatternRam;
use crate::{Byte, Word};

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
}

impl WaveDescription {
    const MAXIMUM_LENGTH: Word = 256;

    pub fn new(
        set: bool,
        frequency: u16,
        output_level: WaveOutputLevel,
        wave: WavePatternRam,
        use_length: bool,
        length: Byte,
        should_play: bool,
    ) -> Self {
        let mut value = Self {
            set,
            frequency,
            output_level,
            wave,
            use_length,
            length,
            remaining_steps: 0,
            should_play,
            sample_clock: 0.0,
        };

        value.reload_length(length);

        value
    }

    pub fn exchange(&mut self, other: &Self) {
        self.set = other.set;
        self.frequency = other.frequency;
        self.output_level = other.output_level;
        self.wave = WavePatternRam {
            data: MemorySector::with_data(other.wave.data.data.clone()),
        };
        self.use_length = other.use_length;
        self.length = other.length;

        if other.set && self.remaining_steps == 0 {
            self.remaining_steps = Self::MAXIMUM_LENGTH;
        }

        self.should_play = other.should_play;
        self.sample_clock = 0.0;
    }

    pub fn step_256(&mut self) {
        if self.use_length && self.remaining_steps > 0 {
            self.remaining_steps -= 1;

            if self.remaining_steps == 0 {
                self.should_play = false;
            }
        }
    }

    pub fn calculate_frequency(&self) -> f32 {
        65536_f32 / (2048 - self.frequency) as f32
    }

    pub fn reload_length(&mut self, length: Byte) {
        self.length = length;
        self.remaining_steps = Self::MAXIMUM_LENGTH - length as Word;
    }

    pub fn next_sample_clock(&mut self) -> f32 {
        let value = self.sample_clock;
        self.sample_clock += 1.0;

        value
    }
}

impl Default for WaveDescription {
    fn default() -> Self {
        Self::new(
            true,
            0,
            WaveOutputLevel::Mute,
            WavePatternRam::default(),
            false,
            0xFF,
            false,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stops_when_no_remaining_steps() {
        let mut wd = WaveDescription::new(
            true,
            1,
            WaveOutputLevel::Vol100Percent,
            WavePatternRam::default(),
            true,
            255,
            true,
        );

        assert_eq!(wd.remaining_steps, 1);
        assert_eq!(wd.should_play, true);

        wd.step_256();

        assert_eq!(wd.remaining_steps, 0);
        assert_eq!(wd.should_play, false);
    }
}
