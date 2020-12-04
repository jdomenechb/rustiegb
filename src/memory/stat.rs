use crate::Byte;

#[derive(Clone)]
pub enum STATMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

impl Default for STATMode {
    fn default() -> Self {
        Self::Mode0
    }
}

#[derive(Default)]
pub struct STAT {
    lyc_ly_coincidence: bool,
    mode_2: bool,
    mode_1: bool,
    mode_0: bool,

    // false: not equal to LCDC LY
    // true: LYC = LCDC LY
    coincidence_flag: bool,

    // 0x0 - During H-Blank
    // 0x1 - During V-Blank
    // 0x2 - During Searching OAM-RAM
    // 0x3 - During transferring data to LCD Driver
    mode: STATMode,
}

impl STAT {
    fn mode_number(&self) -> u8 {
        match self.mode {
            STATMode::Mode0 => 0x0,
            STATMode::Mode1 => 0x1,
            STATMode::Mode2 => 0x2,
            STATMode::Mode3 => 0x3,
        }
    }

    pub fn mode(&self) -> STATMode {
        self.mode.clone()
    }

    pub fn set_mode(&mut self, mode: STATMode) {
        self.mode = mode
    }
}

impl From<Byte> for STAT {
    fn from(value: Byte) -> Self {
        Self {
            lyc_ly_coincidence: value & 0b1000000 == 0b1000000,
            mode_2: value & 0b100000 == 0b100000,
            mode_1: value & 0b10000 == 0b10000,
            mode_0: value & 0b1000 == 0b1000,
            coincidence_flag: value & 0b100 == 0b100,

            mode: match value & 0b11 {
                0x00 => STATMode::Mode0,
                0x01 => STATMode::Mode1,
                0x10 => STATMode::Mode2,
                0x11 => STATMode::Mode3,
                _ => panic!("Unrecognized STAT mode"),
            },
        }
    }
}

impl From<&STAT> for Byte {
    fn from(original: &STAT) -> Byte {
        ((original.lyc_ly_coincidence as Byte) << 6)
            | ((original.mode_2 as Byte) << 5)
            | ((original.mode_1 as Byte) << 4)
            | ((original.mode_0 as Byte) << 3)
            | ((original.coincidence_flag as Byte) << 2)
            | original.mode_number()
    }
}
