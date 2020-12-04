use crate::Byte;

#[derive(Default)]
pub struct InterruptFlag {
    p10_13_transition: bool,
    serial_io_transfer_complete: bool,
    timer_overflow: bool,
    lcd_stat: bool,
    vblank: bool,
}

impl InterruptFlag {
    pub fn new() -> InterruptFlag {
        return InterruptFlag {
            p10_13_transition: false,
            serial_io_transfer_complete: false,
            timer_overflow: false,
            lcd_stat: false,
            vblank: false,
        };
    }

    pub fn is_vblank(&self) -> bool {
        self.vblank
    }

    pub fn set_vblank(&mut self, value: bool) {
        self.vblank = value;
    }

    pub fn is_lcd_stat(&self) -> bool {
        self.lcd_stat
    }

    pub fn set_lcd_stat(&mut self, value: bool) {
        self.lcd_stat = value;
    }

    pub fn is_p10_p13_transition(&self) -> bool {
        self.p10_13_transition
    }

    pub fn set_p10_p13_transition(&mut self, value: bool) {
        self.p10_13_transition = value;
    }
}

impl From<Byte> for InterruptFlag {
    fn from(value: Byte) -> Self {
        Self {
            p10_13_transition: value & 0b10000 == 0b10000,
            serial_io_transfer_complete: value & 0b1000 == 0b1000,
            timer_overflow: value & 0b100 == 0b100,
            lcd_stat: value & 0b10 == 0b10,
            vblank: value & 0b1 == 0b1,
        }
    }
}

impl From<&InterruptFlag> for Byte {
    fn from(original: &InterruptFlag) -> Self {
        let value = ((original.p10_13_transition as Byte) << 4)
            | ((original.serial_io_transfer_complete as Byte) << 3)
            | ((original.timer_overflow as Byte) << 2)
            | ((original.lcd_stat as Byte) << 1)
            | (original.vblank as Byte);

        value
    }
}
