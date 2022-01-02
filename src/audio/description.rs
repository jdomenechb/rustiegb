use crate::audio::sweep::Sweep;
use crate::audio::{VolumeEnvelopeDirection, WaveOutputLevel};
use crate::memory::memory_sector::MemorySector;
use crate::memory::wave_pattern_ram::WavePatternRam;
use crate::{Byte, Memory, Word};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Default)]
pub struct VolumeEnvelopeDescription {
    pub initial_volume: Byte,
    pub current_volume: Byte,
    pub direction: VolumeEnvelopeDirection,
    pub period: u8,
    pub period_timer: u8,
}

impl VolumeEnvelopeDescription {
    pub fn new(initial_volume: Byte, direction: VolumeEnvelopeDirection, period: u8) -> Self {
        Self {
            initial_volume,
            current_volume: initial_volume,
            direction,
            period,
            period_timer: period,
        }
    }

    pub fn step_64(&mut self) {
        if self.period == 0 {
            return;
        }

        if self.period_timer != 0 {
            self.period_timer -= 1;

            if self.period_timer == 0 {
                self.period_timer = self.period;

                match self.direction {
                    VolumeEnvelopeDirection::Up => {
                        if self.current_volume < 0xF {
                            self.current_volume += 1;
                        }
                    }
                    VolumeEnvelopeDirection::Down => {
                        if self.current_volume > 0 {
                            self.current_volume -= 1;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default)]
pub struct PulseDescription {
    pub set: bool,
    pub frequency: Word,
    pub wave_duty_percent: f32,
    pub volume_envelope: VolumeEnvelopeDescription,
    sweep: Option<Sweep>,
    pub stop: bool,
    use_length: bool,
    length: Byte,
    remaining_steps: Byte,
    sample_clock: f32,
}

impl PulseDescription {
    pub fn new(
        set: bool,
        frequency: Word,
        wave_duty_percent: f32,
        volume_envelope: VolumeEnvelopeDescription,
        sweep: Option<Sweep>,
        use_length: bool,
        length: Byte,
    ) -> Self {
        let mut value = Self {
            set,
            frequency,
            wave_duty_percent,
            volume_envelope,
            sweep,
            stop: false,
            use_length,
            length,
            remaining_steps: 0,
            sample_clock: 0.0,
        };

        value.reload_length(length);

        if let Some(mut s) = sweep {
            s.check_first_calculate_new_frequency(&mut value);
            value.sweep = Some(s);
        }

        value
    }

    pub fn step_128(&mut self, memory: Arc<RwLock<Memory>>) {
        if let Some(mut sweep) = self.sweep {
            sweep.step_128(memory, self);
            self.sweep = Some(sweep);
        }
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

    pub fn exchange(&mut self, other: &Self) {
        self.set = other.set;
        self.frequency = other.frequency;
        self.wave_duty_percent = other.wave_duty_percent;
        self.volume_envelope = VolumeEnvelopeDescription::new(
            other.volume_envelope.initial_volume,
            other.volume_envelope.direction,
            other.volume_envelope.period,
        );
        self.sweep = other.sweep;
        self.stop = other.stop;
        self.use_length = other.use_length;
        self.length = other.length;
        self.remaining_steps = other.remaining_steps;
        self.sample_clock = 0.0;
    }

    pub fn calculate_frequency(&self) -> f32 {
        131072_f32 / (2048.0 - self.frequency as f32)
    }

    pub fn reload_length(&mut self, length: Byte) {
        self.length = length;
        self.remaining_steps = 64 - length;
    }

    pub fn reload_sweep(&mut self, sweep: Option<Sweep>) {
        if let Some(s) = sweep {
            if let Some(mut s2) = self.sweep {
                if s2.negate_is_disabled_after_calculation(&s) {
                    self.stop = true;
                }

                s2.exchange(&s);
                self.sweep = Some(s2);
            }
        }
    }

    pub fn next_sample_clock(&mut self) -> f32 {
        let value = self.sample_clock;
        self.sample_clock += 1.0;

        value
    }
}

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
        self.remaining_steps = other.remaining_steps;
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
        self.remaining_steps = 256 - length as Word;
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
    fn test_pdesc_stops_when_no_remaining_steps() {
        let mut pd = PulseDescription::new(
            true,
            1,
            0.0,
            VolumeEnvelopeDescription::default(),
            None,
            true,
            63,
        );

        assert_eq!(pd.remaining_steps, 1);
        assert_eq!(pd.stop, false);

        pd.step_256();

        assert_eq!(pd.remaining_steps, 0);
        assert_eq!(pd.stop, true);
    }

    #[test]
    fn test_wdesc_stops_when_no_remaining_steps() {
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
