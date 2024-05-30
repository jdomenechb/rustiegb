use crate::Byte;

#[derive(Default)]
#[readonly::make]
pub struct InterruptEnable {
    rest: u8,
    pub p10_13_transition: bool,
    serial_io_transfer_complete: bool,
    pub timer_overflow: bool,
    pub lcd_stat: bool,
    pub vblank: bool,
}

impl InterruptEnable {
    pub fn set_vblank(&mut self, value: bool) {
        self.vblank = value;
    }

    pub fn set_lcd_stat(&mut self, value: bool) {
        self.lcd_stat = value;
    }

    pub fn set_p10_p13_transition(&mut self, value: bool) {
        self.p10_13_transition = value;
    }

    pub fn set_timer_overflow(&mut self, value: bool) {
        self.timer_overflow = value;
    }
}

impl From<Byte> for InterruptEnable {
    fn from(value: Byte) -> Self {
        Self {
            rest: value >> 5,
            p10_13_transition: value & 0b10000 == 0b10000,
            serial_io_transfer_complete: value & 0b1000 == 0b1000,
            timer_overflow: value & 0b100 == 0b100,
            lcd_stat: value & 0b10 == 0b10,
            vblank: value & 0b1 == 0b1,
        }
    }
}

impl From<&InterruptEnable> for Byte {
    fn from(original: &InterruptEnable) -> Self {
        original.rest << 5
            | ((original.p10_13_transition as Byte) << 4)
            | ((original.serial_io_transfer_complete as Byte) << 3)
            | ((original.timer_overflow as Byte) << 2)
            | ((original.lcd_stat as Byte) << 1)
            | (original.vblank as Byte)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok() {
        for number in 0..=0xFF {
            let item = InterruptEnable::from(number);

            assert_eq!(Byte::from(&item), number);
        }
    }
}
