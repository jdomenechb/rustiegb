use crate::audio::description::PulseDescription;
use crate::Byte;

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
    remaining_time: Byte,
}

impl Sweep {
    pub fn step_128(&mut self, pulse_description: &mut PulseDescription) {
        if self.remaining_time == 0 {
            return;
        }

        if self.remaining_time > 0 {
            self.remaining_time -= 1;
        }

        if self.remaining_time == 0 {
            self.remaining_time = if self.time > 0 { self.time } else { 8 };

            let to_add_sub = pulse_description.current_frequency >> self.shifts;

            match self.direction {
                SweepDirection::Add => pulse_description.current_frequency += to_add_sub,
                SweepDirection::Sub => pulse_description.current_frequency -= to_add_sub,
            }

            if pulse_description.current_frequency > 2047 {
                pulse_description.stop = true;
            }
        }
    }
}

impl From<Byte> for Sweep {
    fn from(obj: Byte) -> Self {
        let shifts = obj & 0b111;
        let time = (obj >> 4) & 0b111;

        Self {
            time,
            shifts,
            direction: match obj & 0b1000 == 0b1000 {
                true => SweepDirection::Sub,
                false => SweepDirection::Add,
            },
            remaining_time: time,
        }
    }
}
