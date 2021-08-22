use crate::Byte;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum SweepDirection {
    Add,
    Sub,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Sweep {
    pub time: Byte,
    pub shifts: Byte,
    pub direction: SweepDirection,
    pub remaining_time: Byte,
    pub happened: bool,
}

impl Sweep {
    pub fn step_128(&mut self) {
        self.happened = false;

        if self.remaining_time == 0 {
            return;
        }

        self.remaining_time -= 1;

        if self.remaining_time == 0 {
            self.remaining_time = self.time;

            self.happened = true;
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
            happened: false,
        }
    }
}
