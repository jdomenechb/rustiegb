use crate::audio::pulse::description::PulseDescription;
use crate::audio::registers::{ChannelStopabble, FrequencyUpdatable};
use crate::io::registers::IORegisters;
use crate::{Byte, Word};
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

    pub fn trigger_control_register_update(&mut self, pulse_description: &mut PulseDescription) {
        self.shadow_frequency = pulse_description.get_frequency();
        self.reload_timer();

        self.enabled = self.time > 0 || self.shifts > 0;

        if self.shifts > 0 {
            self.calculate_new_frequency(pulse_description);
        };
    }

    pub fn step_128(
        &mut self,
        io_registers: Arc<RwLock<IORegisters>>,
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
                            io_registers.write().update_audio_1_frequency(new_frequency);
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
            enabled: false,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ok() {
        let sweep = Sweep::from(0b1011010);

        assert_eq!(sweep.time, 0b101);
        assert_eq!(sweep.shifts, 0b010);
        assert_eq!(sweep.direction, SweepDirection::Sub);
        assert_eq!(sweep.timer, 0);
        assert_eq!(sweep.enabled, false);
        assert_eq!(sweep.shadow_frequency, 0);
        assert_eq!(sweep.calculated, false);
    }

    #[test]
    fn test_default_ok() {
        let sweep = Sweep::default();

        assert_eq!(sweep.time, 0);
        assert_eq!(sweep.shifts, 0);
        assert_eq!(sweep.direction, SweepDirection::Add);
        assert_eq!(sweep.timer, 0);
        assert_eq!(sweep.enabled, false);
        assert_eq!(sweep.shadow_frequency, 0);
        assert_eq!(sweep.calculated, false);
    }

    #[test]
    fn test_update_from_register_basic() {
        let mut sweep = Sweep::default();
        sweep.calculated = true;

        sweep.update_from_register(0b1011010, &mut PulseDescription::default());

        assert_eq!(sweep.time, 0b101);
        assert_eq!(sweep.shifts, 0b010);
        assert_eq!(sweep.direction, SweepDirection::Sub);
        assert_eq!(sweep.timer, 0);
        assert_eq!(sweep.enabled, false);
        assert_eq!(sweep.shadow_frequency, 0);
        assert_eq!(sweep.calculated, false);
    }

    #[test]
    fn test_update_from_register_stops_channel() {
        let mut sweep = Sweep::default();
        sweep.calculated = true;
        sweep.direction = SweepDirection::Sub;

        let mut description = PulseDescription::default();
        assert!(!description.stop);

        sweep.update_from_register(0b1010010, &mut description);

        assert_eq!(sweep.time, 0b101);
        assert_eq!(sweep.shifts, 0b010);
        assert_eq!(sweep.direction, SweepDirection::Add);
        assert_eq!(sweep.timer, 0);
        assert_eq!(sweep.enabled, false);
        assert_eq!(sweep.shadow_frequency, 0);
        assert_eq!(sweep.calculated, false);

        assert!(description.stop);
    }
}
