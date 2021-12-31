use crate::audio::sweep::Sweep;
use crate::audio::{VolumeEnvelopeDirection, WaveOutputLevel};
use crate::memory::memory_sector::MemorySector;
use crate::memory::wave_pattern_ram::WavePatternRam;
use crate::{Byte, Word};

pub struct PulseDescription {
    pub pulse_n: u8,
    pub current_frequency: Word,
    pub wave_duty_percent: f32,
    pub initial_volume_envelope: Byte,
    pub volume_envelope: Byte,
    pub volume_envelope_direction: VolumeEnvelopeDirection,
    pub volume_envelope_duration_in_1_64_s: u8,
    pub remaining_volume_envelope_duration_in_1_64_s: u8,
    pub sweep: Option<Sweep>,
    pub stop: bool,
    pub use_length: bool,
    pub length: Byte,
    pub remaining_steps: Byte,
}

impl PulseDescription {
    pub fn step_128(&mut self) {
        if let Some(mut sweep) = self.sweep {
            sweep.step_128(self);
            self.sweep = Some(sweep);
        }
    }

    pub fn step_64(&mut self) {
        if self.volume_envelope_duration_in_1_64_s == 0 {
            return;
        }

        if self.remaining_volume_envelope_duration_in_1_64_s == 0 {
            match self.volume_envelope_direction {
                VolumeEnvelopeDirection::Up => {
                    if self.volume_envelope < 0xF {
                        self.volume_envelope += 1;
                    }
                }
                VolumeEnvelopeDirection::Down => {
                    if self.volume_envelope > 0 {
                        self.volume_envelope -= 1;
                    }
                }
            }

            self.remaining_volume_envelope_duration_in_1_64_s =
                self.volume_envelope_duration_in_1_64_s;

            return;
        }

        self.remaining_volume_envelope_duration_in_1_64_s -= 1;
    }

    pub fn step_256(&mut self) {
        if self.use_length {
            if self.remaining_steps > 0 {
                self.remaining_steps -= 1;
            } else {
                self.stop = true;
            }
        }
    }

    pub fn exchange(&mut self, other: &Self) {
        self.pulse_n = other.pulse_n;
        self.current_frequency = other.current_frequency;
        self.wave_duty_percent = other.wave_duty_percent;
        self.initial_volume_envelope = other.initial_volume_envelope;
        self.volume_envelope = other.volume_envelope;
        self.volume_envelope_direction = other.volume_envelope_direction;
        self.volume_envelope_duration_in_1_64_s = other.volume_envelope_duration_in_1_64_s;
        self.remaining_volume_envelope_duration_in_1_64_s =
            other.remaining_volume_envelope_duration_in_1_64_s;
        self.sweep = other.sweep;
        self.stop = false;
        self.use_length = other.use_length;
        self.length = other.length;
    }

    pub fn calculate_frequency(&self) -> f32 {
        131072_f32 / (2048.0 - self.current_frequency as f32)
    }
}

impl PartialEq for PulseDescription {
    fn eq(&self, other: &Self) -> bool {
        other.pulse_n == self.pulse_n
            && other.current_frequency == self.current_frequency
            && other.wave_duty_percent == self.wave_duty_percent
            && other.initial_volume_envelope == self.initial_volume_envelope
            && other.volume_envelope_direction == self.volume_envelope_direction
            && other.volume_envelope_duration_in_1_64_s == self.volume_envelope_duration_in_1_64_s
            && other.sweep == self.sweep
            && other.use_length == other.use_length
            && other.length == other.length
    }
}

impl Default for PulseDescription {
    fn default() -> Self {
        Self {
            pulse_n: 0,
            current_frequency: 0,
            wave_duty_percent: 0.0,
            initial_volume_envelope: 0,
            volume_envelope: 0,
            volume_envelope_direction: VolumeEnvelopeDirection::Up,
            volume_envelope_duration_in_1_64_s: 0,
            remaining_volume_envelope_duration_in_1_64_s: 0,
            sweep: None,
            stop: false,
            use_length: false,
            length: 0,
            remaining_steps: 0,
        }
    }
}

pub struct WaveDescription {
    pub frequency: u16,
    pub output_level: WaveOutputLevel,
    pub wave: WavePatternRam,
    pub use_length: bool,
    pub length: Byte,
    pub remaining_steps: Word,
    pub should_play: bool,
}

impl WaveDescription {
    pub fn new(
        frequency: u16,
        output_level: WaveOutputLevel,
        wave: WavePatternRam,
        use_length: bool,
        length: Byte,
        should_play: bool,
    ) -> Self {
        Self {
            frequency,
            output_level,
            wave,
            use_length,
            length,
            remaining_steps: 256 - length as Word,
            should_play,
        }
    }

    pub fn exchange(&mut self, other: &Self) {
        self.frequency = other.frequency;
        self.output_level = other.output_level;
        self.wave = WavePatternRam {
            data: MemorySector::with_data(other.wave.data.data.clone()),
        };
        self.use_length = other.use_length;
        self.remaining_steps = other.remaining_steps;
        self.should_play = other.should_play;
    }

    pub fn step_256(&mut self) {
        if self.use_length {
            if self.remaining_steps > 0 {
                self.remaining_steps -= 1;
            } else {
                self.should_play = false;
            }
        }
    }

    pub fn calculate_frequency(&self) -> f32 {
        65536_f32 / (2048 - self.frequency) as f32
    }
}

impl PartialEq for WaveDescription {
    fn eq(&self, other: &Self) -> bool {
        other.frequency == self.frequency
            && other.output_level == self.output_level
            && other.use_length == self.use_length
            && other.length == self.length
            && other.should_play == self.should_play
            && other.wave.data.data == self.wave.data.data
    }
}

impl Default for WaveDescription {
    fn default() -> Self {
        Self::new(
            0,
            WaveOutputLevel::Mute,
            WavePatternRam::default(),
            false,
            0xFF,
            false,
        )
    }
}
