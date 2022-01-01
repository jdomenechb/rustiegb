use crate::audio::sweep::Sweep;
use crate::audio::{VolumeEnvelopeDirection, WaveOutputLevel};
use crate::memory::memory_sector::MemorySector;
use crate::memory::wave_pattern_ram::WavePatternRam;
use crate::{Byte, Word};

#[derive(Default)]
pub struct VolumeEnvelopeDescription {
    pub volume_envelope: Byte,
    pub volume_envelope_direction: VolumeEnvelopeDirection,
    pub volume_envelope_duration_in_1_64_s: u8,
    pub remaining_volume_envelope_duration_in_1_64_s: u8,
}

impl VolumeEnvelopeDescription {
    pub fn new(
        volume_envelope: Byte,
        volume_envelope_direction: VolumeEnvelopeDirection,
        volume_envelope_duration_in_1_64_s: u8,
    ) -> Self {
        Self {
            volume_envelope,
            volume_envelope_direction,
            volume_envelope_duration_in_1_64_s,
            remaining_volume_envelope_duration_in_1_64_s: volume_envelope_duration_in_1_64_s,
        }
    }
}

#[derive(Default)]
pub struct PulseDescription {
    pub current_frequency: Word,
    pub wave_duty_percent: f32,
    pub volume_envelope: VolumeEnvelopeDescription,
    sweep: Option<Sweep>,
    pub stop: bool,
    use_length: bool,
    length: Byte,
    remaining_steps: Byte,
}

impl PulseDescription {
    pub fn new(
        frequency: Word,
        wave_duty_percent: f32,
        volume_envelope: VolumeEnvelopeDescription,
        sweep: Option<Sweep>,
        use_length: bool,
        length: Byte,
    ) -> Self {
        let mut value = Self {
            current_frequency: frequency,
            wave_duty_percent,
            volume_envelope,
            sweep,
            stop: false,
            use_length,
            length,
            remaining_steps: 0,
        };

        value.reload_length(length);

        value
    }

    pub fn step_128(&mut self) {
        if let Some(mut sweep) = self.sweep {
            sweep.step_128(self);
            self.sweep = Some(sweep);
        }
    }

    pub fn step_64(&mut self) {
        if self.volume_envelope.volume_envelope_duration_in_1_64_s == 0 {
            return;
        }

        if self
            .volume_envelope
            .remaining_volume_envelope_duration_in_1_64_s
            == 0
        {
            match self.volume_envelope.volume_envelope_direction {
                VolumeEnvelopeDirection::Up => {
                    if self.volume_envelope.volume_envelope < 0xF {
                        self.volume_envelope.volume_envelope += 1;
                    }
                }
                VolumeEnvelopeDirection::Down => {
                    if self.volume_envelope.volume_envelope > 0 {
                        self.volume_envelope.volume_envelope -= 1;
                    }
                }
            }

            self.volume_envelope
                .remaining_volume_envelope_duration_in_1_64_s =
                self.volume_envelope.volume_envelope_duration_in_1_64_s;

            return;
        }

        self.volume_envelope
            .remaining_volume_envelope_duration_in_1_64_s -= 1;
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
        self.current_frequency = other.current_frequency;
        self.wave_duty_percent = other.wave_duty_percent;
        self.volume_envelope = VolumeEnvelopeDescription::new(
            other.volume_envelope.volume_envelope,
            other.volume_envelope.volume_envelope_direction,
            other.volume_envelope.volume_envelope_duration_in_1_64_s,
        );
        self.sweep = other.sweep;
        self.stop = other.stop;
        self.use_length = other.use_length;
        self.length = other.length;
        self.remaining_steps = other.remaining_steps;
    }

    pub fn calculate_frequency(&self) -> f32 {
        131072_f32 / (2048.0 - self.current_frequency as f32)
    }

    pub fn reload_length(&mut self, length: Byte) {
        self.length = length;
        self.remaining_steps = 64 - length;
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
        let mut value = Self {
            frequency,
            output_level,
            wave,
            use_length,
            length,
            remaining_steps: 0,
            should_play,
        };

        value.reload_length(length);

        value
    }

    pub fn exchange(&mut self, other: &Self) {
        self.frequency = other.frequency;
        self.output_level = other.output_level;
        self.wave = WavePatternRam {
            data: MemorySector::with_data(other.wave.data.data.clone()),
        };
        self.use_length = other.use_length;
        self.length = other.length;
        self.remaining_steps = other.remaining_steps;
        self.should_play = other.should_play;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdesc_stops_when_no_remaining_steps() {
        let mut pd =
            PulseDescription::new(1, 0.0, VolumeEnvelopeDescription::default(), None, true, 63);

        assert_eq!(pd.remaining_steps, 1);
        assert_eq!(pd.stop, false);

        pd.step_256();

        assert_eq!(pd.remaining_steps, 0);
        assert_eq!(pd.stop, true);
    }

    #[test]
    fn test_wdesc_stops_when_no_remaining_steps() {
        let mut wd = WaveDescription::new(
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
