use crate::audio::pulse::description::PulseDescription;
use crate::audio::registers::{ChannelStopabble, FrequencyUpdatable};
use crate::{Byte, Memory, Word};
use direction::SweepDirection;
use parking_lot::RwLock;
use std::sync::Arc;

mod direction;

#[derive(Copy, Clone)]
pub struct Sweep {
    time: Byte,
    shifts: Byte,
    direction: SweepDirection,
    timer: Byte,
    enabled: bool,
    shadow_frequency: Word,
    calculated: bool,
}

impl Sweep {
    pub fn step_128(
        &mut self,
        memory: Arc<RwLock<Memory>>,
        pulse_description: &mut PulseDescription,
    ) {
        if self.timer > 0 {
            self.timer -= 1;

            if self.timer == 0 {
                self.reload_timer();

                if self.enabled && self.time > 0 {
                    let new_frequency = self.calculate_new_frequency(pulse_description);

                    if new_frequency < 2048 && self.shifts > 0 {
                        self.shadow_frequency = new_frequency;
                        pulse_description.frequency = new_frequency;

                        {
                            memory.write().update_audio_1_frequency(new_frequency);
                        }

                        self.calculate_new_frequency(pulse_description);
                    }
                }
            }
        }
    }

    pub fn calculate_new_frequency(&mut self, pulse_description: &mut PulseDescription) -> u16 {
        self.calculated = true;

        let to_add_sub = self.shadow_frequency >> self.shifts;

        let new_frequency = match self.direction {
            SweepDirection::Add => self.shadow_frequency.wrapping_add(to_add_sub),
            SweepDirection::Sub => self.shadow_frequency.wrapping_sub(to_add_sub),
        };

        if new_frequency > 2047 {
            pulse_description.stop_channel();
        }

        new_frequency
    }

    pub fn reload_timer(&mut self) {
        self.timer = if self.time > 0 { self.time } else { 8 };
    }

    pub fn trigger_control_register_update(&mut self, pulse_description: &mut PulseDescription) {
        self.shadow_frequency = pulse_description.get_frequency();
        self.reload_timer();

        self.enabled = self.time > 0 || self.shifts > 0;

        if self.shifts > 0 {
            self.calculate_new_frequency(pulse_description);
        };
    }

    pub fn update_from_register(
        &mut self,
        register: Byte,
        pulse_description: &mut PulseDescription,
    ) {
        let shifts = register & 0b111;
        let time = (register >> 4) & 0b111;

        let direction = match register & 0b1000 == 0b1000 {
            true => SweepDirection::Sub,
            false => SweepDirection::Add,
        };

        if self.calculated
            && self.direction == SweepDirection::Sub
            && direction == SweepDirection::Add
        {
            pulse_description.stop_channel();
        }

        self.time = time;
        self.shifts = shifts;
        self.direction = direction;

        self.calculated = false;
    }
}

impl From<Byte> for Sweep {
    fn from(register: Byte) -> Self {
        let shifts = register & 0b111;
        let time = (register >> 4) & 0b111;

        Self {
            time,
            shifts,
            direction: match register & 0b1000 == 0b1000 {
                true => SweepDirection::Sub,
                false => SweepDirection::Add,
            },
            timer: 0,
            enabled: true,
            shadow_frequency: 0,
            calculated: false,
        }
    }
}

impl Default for Sweep {
    fn default() -> Self {
        Sweep::from(0x80)
    }
}
