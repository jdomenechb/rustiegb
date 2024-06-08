use crate::Byte;

#[derive(Clone)]
pub enum STATMode {
    HBlank,
    VBlank,
    SearchOamRam,
    LCDTransfer,
}

impl Default for STATMode {
    fn default() -> Self {
        Self::HBlank
    }
}

#[derive(Default)]
pub struct Stat {
    pub lyc_ly_coincidence: bool,
    pub mode_2: bool,
    pub mode_1: bool,
    pub mode_0: bool,

    // false: not equal to LCDC LY
    // true: LYC = LCDC LY
    pub coincidence_flag: bool,

    // 0x0 - During H-Blank
    // 0x1 - During V-Blank
    // 0x2 - During Searching OAM-RAM
    // 0x3 - During transferring data to LCD Driver
    mode: STATMode,
}

impl Stat {
    fn mode_number(&self) -> u8 {
        match self.mode {
            STATMode::HBlank => 0x0,
            STATMode::VBlank => 0x1,
            STATMode::SearchOamRam => 0x2,
            STATMode::LCDTransfer => 0x3,
        }
    }

    pub fn mode(&self) -> STATMode {
        self.mode.clone()
    }

    pub fn set_mode(&mut self, mode: STATMode) {
        self.mode = mode
    }

    pub fn update(&mut self, value: Byte) {
        self.lyc_ly_coincidence = value & 0b1000000 == 0b1000000;
        self.mode_2 = value & 0b100000 == 0b100000;
        self.mode_1 = value & 0b10000 == 0b10000;
        self.mode_0 = value & 0b1000 == 0b1000;
        self.coincidence_flag = value & 0b100 == 0b100;

        self.mode = match value & 0b11 {
            0b00 => STATMode::HBlank,
            0b01 => STATMode::VBlank,
            0b10 => STATMode::SearchOamRam,
            0b11 => STATMode::LCDTransfer,
            _ => panic!("Unrecognized STAT mode"),
        };
    }
}

impl From<&Stat> for Byte {
    fn from(original: &Stat) -> Byte {
        0b10000000
            | ((original.lyc_ly_coincidence as Byte) << 6)
            | ((original.mode_2 as Byte) << 5)
            | ((original.mode_1 as Byte) << 4)
            | ((original.mode_0 as Byte) << 3)
            | ((original.coincidence_flag as Byte) << 2)
            | original.mode_number()
    }
}

#[cfg(test)]
mod tests {
    use crate::io::stat::Stat;
    use crate::Byte;

    #[test]
    fn test_ok() {
        let mut item = Stat::default();

        for number in 0..=0b1111111 {
            item.update(number);

            assert_eq!(Byte::from(&item), number | 0b10000000);
        }
    }
}
