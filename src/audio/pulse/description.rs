use crate::audio::pulse::sweep::Sweep;
use crate::audio::pulse::PulseWavePatternDuty;
use crate::audio::volume_envelope::VolumeEnvelopeDescription;
use crate::{Byte, Memory, Word};
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
    remaining_steps: Byte,
    sample_clock: f32,
}

impl PulseDescription {
    const MAXIMUM_LENGTH: Byte = 64;

    pub fn new(
        set: bool,
        frequency: Word,
        wave_duty: PulseWavePatternDuty,
        volume_envelope: VolumeEnvelopeDescription,
        sweep: Option<Sweep>,
        use_length: bool,
        length: Byte,
    ) -> Self {
        let mut value = Self {
            set,
            frequency,
            wave_duty,
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
        self.wave_duty = other.wave_duty.clone();
        self.volume_envelope = VolumeEnvelopeDescription::new(
            other.volume_envelope.initial_volume,
            other.volume_envelope.direction,
            other.volume_envelope.period,
        );
        self.sweep = other.sweep;
        self.stop = other.stop;
        self.use_length = other.use_length;
        self.length = other.length;

        if other.set && self.remaining_steps == 0 {
            self.remaining_steps = Self::MAXIMUM_LENGTH;
        }

        self.sample_clock = 0.0;
    }

    pub fn calculate_frequency(&self) -> f32 {
        131072_f32 / (2048.0 - self.frequency as f32)
    }

    pub fn reload_length(&mut self, length: Byte) {
        self.length = length;
        self.remaining_steps = Self::MAXIMUM_LENGTH - length;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stops_when_no_remaining_steps() {
        let mut pd = PulseDescription::new(
            true,
            1,
            PulseWavePatternDuty::default(),
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
}
