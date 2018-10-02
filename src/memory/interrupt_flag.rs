pub struct InterruptFlag {
    p10_13_transition: bool,
    serial_io_transfer_complete: bool,
    timer_overflow: bool,
    lcdc: bool,
    vblank: bool
}

impl InterruptFlag {
    pub fn new() -> InterruptFlag {
        return InterruptFlag {
            p10_13_transition: false,
            serial_io_transfer_complete: false,
            timer_overflow: false,
            lcdc: false,
            vblank: false
        }
    }

    pub fn from_u8(&mut self, value: u8) {
        self.p10_13_transition = value & 0b10000 == 0b10000;
        self.serial_io_transfer_complete = value & 0b1000 == 0b1000;
        self.timer_overflow = value & 0b100 == 0b100;
        self.lcdc = value & 0b10 == 0b10;
        self.vblank = value & 0b1 == 0b1;
    }
}