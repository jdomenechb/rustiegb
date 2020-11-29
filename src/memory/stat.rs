#[derive(Default)]
pub struct STAT {
    // TODO: Other bits

    // 0 - During H-Blank
    // 1 - During V-Blank
    // 2 - During Searching OAM-RAM
    // 3 - During transferring data to LCD Driver
    pub mode: u8,
}

impl STAT {
    pub fn new() -> STAT {
        return STAT {
            mode: 0,
        }
    }

    pub fn from_u8(&mut self, value: u8) {
        self.mode = value & 0x11;
    }

    pub fn to_u8(&self) -> u8 {
        return self.mode;
    }
}