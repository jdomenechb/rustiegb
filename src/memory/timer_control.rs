pub struct TimerControl {
    started: bool,
    period_microseconds: f64,
    frecuency_option: u8
}

impl TimerControl {
    pub fn new() -> TimerControl {
        return TimerControl {
            started: false,
            period_microseconds: 0.0,
            frecuency_option: 0
        }
    }

    pub fn from_u8(&mut self, value: u8) {
        self.started = value & 0x4 == 0x4;
        self.frecuency_option = value & 0x3;

        match self.frecuency_option {
            0 => self.period_microseconds = 1.0 / 4096.0 * 1000000.0,
            1 => self.period_microseconds = 1.0 / 262144.0 * 1000000.0,
            2 => self.period_microseconds = 1.0 / 65536.0 * 1000000.0,
            3 => self.period_microseconds = 1.0 / 16384.0 * 1000000.0,
            _ => panic!("Not recognized timer control option: {:X}", self.frecuency_option)
        } 
    }
}