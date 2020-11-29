use crate::Byte;

#[derive(Default)]
pub struct STAT {
    // TODO: Other bits

    // 0 - During H-Blank
    // 1 - During V-Blank
    // 2 - During Searching OAM-RAM
    // 3 - During transferring data to LCD Driver
    pub mode: Byte,
}

// TODO: Move to From<> impl
impl STAT {
    pub fn new() -> STAT {
        return STAT { mode: 0 };
    }

    pub fn from_byte(&mut self, value: Byte) {
        self.mode = value & 0b11;
    }

    pub fn to_byte(&self) -> Byte {
        return self.mode;
    }
}
