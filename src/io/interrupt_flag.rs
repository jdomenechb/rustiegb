use crate::Byte;

#[derive(Default)]
#[readonly::make]
pub struct InterruptFlag {
    pub p10_13_transition: bool,
    serial_io_transfer_complete: bool,
    pub timer_overflow: bool,
    pub lcd_stat: bool,
    pub vblank: bool,
}

impl InterruptFlag {
    pub fn new() -> InterruptFlag {
        InterruptFlag {
            p10_13_transition: false,
            serial_io_transfer_complete: false,
            timer_overflow: false,
            lcd_stat: false,
            vblank: false,
        }
    }

    pub fn update(&mut self, value: Byte) {
        self.p10_13_transition = value & 0b10000 == 0b10000;
        self.serial_io_transfer_complete = value & 0b1000 == 0b1000;
        self.timer_overflow = value & 0b100 == 0b100;
        self.lcd_stat = value & 0b10 == 0b10;
        self.vblank = value & 0b1 == 0b1;
    }

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

impl From<&InterruptFlag> for Byte {
    fn from(original: &InterruptFlag) -> Self {
        0b11100000
            | ((original.p10_13_transition as Byte) << 4)
            | ((original.serial_io_transfer_complete as Byte) << 3)
            | ((original.timer_overflow as Byte) << 2)
            | ((original.lcd_stat as Byte) << 1)
            | (original.vblank as Byte)
    }
}

#[cfg(test)]
mod tests {
    use crate::Byte;
    use crate::io::interrupt_flag::InterruptFlag;

    #[test]
    fn test_ok() {
        for number in 0..=0b11111 {
            let mut item = InterruptFlag::new();
            item.update(number);

            assert_eq!(Byte::from(&item), number | 0b11100000);
        }
    }
}
