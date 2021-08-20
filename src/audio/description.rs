use crate::audio::{VolumeEnvelopeDirection, WaveOutputLevel};
use crate::memory::memory_sector::MemorySector;
use crate::memory::wave_pattern_ram::WavePatternRam;
use crate::Byte;

pub struct PulseDescription {
    pub pulse_n: u8,
    pub frequency: f32,
    pub wave_duty_percent: f32,
    pub initial_volume_envelope: Byte,
    pub volume_envelope: Byte,
    pub volume_envelope_direction: VolumeEnvelopeDirection,
    pub volume_envelope_duration_in_1_64_s: u8,
    pub remaining_volume_envelope_duration_in_1_64_s: u8,
}

impl PulseDescription {
    pub fn step_64(&mut self) {
        if self.volume_envelope_duration_in_1_64_s == 0 {
            return;
        }

        if self.remaining_volume_envelope_duration_in_1_64_s == 0 {
            match self.volume_envelope_direction {
                VolumeEnvelopeDirection::UP => {
                    if self.volume_envelope < 0xF {
                        self.volume_envelope += 1;
                    }
                }
                VolumeEnvelopeDirection::DOWN => {
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

        return;
    }

    pub fn exchange(&mut self, other: &Self) {
        self.pulse_n = other.pulse_n;
        self.frequency = other.frequency;
        self.wave_duty_percent = other.wave_duty_percent;
        self.initial_volume_envelope = other.initial_volume_envelope;
        self.volume_envelope = other.volume_envelope;
        self.volume_envelope_direction = other.volume_envelope_direction;
        self.volume_envelope_duration_in_1_64_s = other.volume_envelope_duration_in_1_64_s;
        self.remaining_volume_envelope_duration_in_1_64_s =
            other.remaining_volume_envelope_duration_in_1_64_s;
    }
}

impl PartialEq for PulseDescription {
    fn eq(&self, other: &Self) -> bool {
        return other.pulse_n == self.pulse_n
            && other.frequency == self.frequency
            && other.wave_duty_percent == self.wave_duty_percent
            && other.initial_volume_envelope == self.initial_volume_envelope
            && other.volume_envelope_direction == self.volume_envelope_direction
            && other.volume_envelope_duration_in_1_64_s == self.volume_envelope_duration_in_1_64_s
            && other.remaining_volume_envelope_duration_in_1_64_s
                == self.remaining_volume_envelope_duration_in_1_64_s;
    }
}

impl Default for PulseDescription {
    fn default() -> Self {
        Self {
            pulse_n: 0,
            frequency: 0.0,
            wave_duty_percent: 0.0,
            initial_volume_envelope: 0,
            volume_envelope: 0,
            volume_envelope_direction: VolumeEnvelopeDirection::UP,
            volume_envelope_duration_in_1_64_s: 0,
            remaining_volume_envelope_duration_in_1_64_s: 0,
        }
    }
}

pub struct WaveDescription {
    pub frequency: f32,
    pub output_level: WaveOutputLevel,
    pub wave: WavePatternRam,
}

impl WaveDescription {
    pub fn exchange(&mut self, other: &Self) {
        self.frequency = other.frequency;
        self.output_level = other.output_level.clone();
        self.wave = WavePatternRam {
            data: MemorySector::with_data(other.wave.data.data.clone()),
        };
    }
}

impl PartialEq for WaveDescription {
    fn eq(&self, other: &Self) -> bool {
        return other.frequency == self.frequency && other.output_level == self.output_level;
    }
}

impl Default for WaveDescription {
    fn default() -> Self {
        Self {
            frequency: 0.0,
            output_level: WaveOutputLevel::Mute,
            wave: WavePatternRam::default(),
        }
    }
}
