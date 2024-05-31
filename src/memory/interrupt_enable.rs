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

    pub value: Byte,
}

impl InterruptEnable {
    pub fn set_vblank(&mut self, value: bool) {
        self.vblank = value;
        self.update_internal_value();
    }

    pub fn set_lcd_stat(&mut self, value: bool) {
        self.lcd_stat = value;
        self.update_internal_value();
    }

    pub fn set_p10_p13_transition(&mut self, value: bool) {
        self.p10_13_transition = value;
        self.update_internal_value();
    }

    pub fn set_timer_overflow(&mut self, value: bool) {
        self.timer_overflow = value;
        self.update_internal_value();
    }

    pub fn update(&mut self, value: Byte) {
        self.rest = value >> 5;
        self.p10_13_transition = value & 0b10000 == 0b10000;
        self.serial_io_transfer_complete = value & 0b1000 == 0b1000;
        self.timer_overflow = value & 0b100 == 0b100;
        self.lcd_stat = value & 0b10 == 0b10;
        self.vblank = value & 0b1 == 0b1;
    }

    fn update_internal_value(&mut self) {
        self.value = self.rest << 5
            | ((self.p10_13_transition as Byte) << 4)
            | ((self.serial_io_transfer_complete as Byte) << 3)
            | ((self.timer_overflow as Byte) << 2)
            | ((self.lcd_stat as Byte) << 1)
            | (self.vblank as Byte);
    }
}

impl From<&InterruptEnable> for Byte {
    fn from(original: &InterruptEnable) -> Self {
        original.value
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
