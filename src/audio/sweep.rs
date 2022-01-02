use crate::audio::description::PulseDescription;
use crate::{Byte, Memory, Word};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum SweepDirection {
    Add,
    Sub,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Sweep {
    time: Byte,
    shifts: Byte,
    direction: SweepDirection,
    timer: Byte,
    enabled: bool,
    shadow_frequency: Word,
}

impl Sweep {
    pub fn new(sweep_register: Byte, frequency: Word) -> Self {
        let shifts = sweep_register & 0b111;
        let time = (sweep_register >> 4) & 0b111;

        Self {
            time,
            shifts,
            direction: match sweep_register & 0b1000 == 0b1000 {
                true => SweepDirection::Sub,
                false => SweepDirection::Add,
            },
            timer: if time > 0 { time } else { 8 },
            enabled: time > 0 || shifts > 0,
            shadow_frequency: frequency,
        }
    }

    pub fn step_128(
        &mut self,
        memory: Arc<RwLock<Memory>>,
        pulse_description: &mut PulseDescription,
    ) {
        if self.timer == 0 {
            return;
        }

        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer == 0 {
            self.timer = if self.time > 0 { self.time } else { 8 };

            if self.enabled && self.time > 0 {
                let new_frequency = self.calculate_new_frequency(pulse_description);

                if new_frequency < 2048 && self.shifts > 0 {
                    self.shadow_frequency = new_frequency;
                    pulse_description.current_frequency = new_frequency;

                    {
                        memory.write().update_audio_1_frequency(new_frequency);
                    }

                    self.calculate_new_frequency(pulse_description);
                }
            }
        }
    }

    pub fn calculate_new_frequency(&mut self, pulse_description: &mut PulseDescription) -> u16 {
        let to_add_sub = self.shadow_frequency >> self.shifts;

        let new_frequency = match self.direction {
            SweepDirection::Add => self.shadow_frequency.wrapping_add(to_add_sub),
            SweepDirection::Sub => self.shadow_frequency.wrapping_add(to_add_sub),
        };

        if new_frequency > 2047 {
            pulse_description.stop = true;
        }

        new_frequency
    }

    pub fn has_shifts(&self) -> bool {
        self.shifts > 0
    }
}
