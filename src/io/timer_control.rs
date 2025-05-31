use crate::Byte;

#[readonly::make]
#[derive(Default, Debug)]
pub struct TimerControl {
    pub started: bool,
    input_clock_select: u8,
}

impl TimerControl {
    pub fn update(&mut self, value: Byte) {
        self.started = value & 0b100 == 0b100;
        self.input_clock_select = value & 0b11;
    }

    pub fn get_divider(&self) -> u32 {
        match self.input_clock_select {
            0 => 1024,
            1 => 16,
            2 => 64,
            3 => 256,
            _ => panic!("Invalid input clock select"),
        }
    }
}
