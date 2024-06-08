use crate::Byte;

#[readonly::make]
#[derive(Default)]
pub struct TimerControl {
    pub started: bool,
    pub input_clock_select: u8,
}

impl From<Byte> for TimerControl {
    fn from(value: u8) -> Self {
        Self {
            started: value & 0b100 == 0b100,
            input_clock_select: value & 0b11,
        }
    }
}
