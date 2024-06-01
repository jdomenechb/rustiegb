use crate::utils::math::set_bit;
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
        self.value = set_bit(self.value, 0, value);
    }

    pub fn set_lcd_stat(&mut self, value: bool) {
        self.lcd_stat = value;
        self.value = set_bit(self.value, 1, value);
    }

    pub fn set_timer_overflow(&mut self, value: bool) {
        self.timer_overflow = value;
        self.value = set_bit(self.value, 2, value);
    }

    pub fn set_p10_p13_transition(&mut self, value: bool) {
        self.p10_13_transition = value;
        self.value = set_bit(self.value, 4, value);
    }

    pub fn update(&mut self, value: Byte) {
        self.rest = value >> 5;
        self.p10_13_transition = value & 0b1_0000 == 0b1_0000;
        self.serial_io_transfer_complete = value & 0b1000 == 0b1000;
        self.timer_overflow = value & 0b100 == 0b100;
        self.lcd_stat = value & 0b10 == 0b10;
        self.vblank = value & 0b1 == 0b1;

        self.value = value;
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
    use test_case::test_case;

    #[test]
    fn it_can_be_updated() {
        let mut item = InterruptEnable::default();

        for number in 0..=0xFF {
            item.update(number);

            assert_eq!(Byte::from(&item), number);
        }
    }

    #[test_case(0x00, false, 0x00; "false to false")]
    #[test_case(0x00, true, 0x01; "false to true")]
    #[test_case(0xFF, false, 0b1111_1110; "true to false")]
    #[test_case(0xFF, true, 0xFF; "true to true")]
    fn it_can_set_vblank(init: Byte, value: bool, expected: Byte) {
        let mut ie = InterruptEnable::default();
        ie.update(init);

        ie.set_vblank(value);

        assert_eq!(Byte::from(&ie), expected);
    }

    #[test_case(0x00, false, 0x00; "false to false")]
    #[test_case(0x00, true, 0b0000_0010; "false to true")]
    #[test_case(0xFF, false, 0b_1111_1101; "true to false")]
    #[test_case(0xFF, true, 0xFF; "true to true")]
    fn it_can_set_lcd_stat(init: Byte, value: bool, expected: Byte) {
        let mut ie = InterruptEnable::default();
        ie.update(init);

        ie.set_lcd_stat(value);

        assert_eq!(Byte::from(&ie), expected);
    }

    #[test_case(0x00, false, 0x00; "false to false")]
    #[test_case(0x00, true, 0b0000_0100; "false to true")]
    #[test_case(0xFF, false, 0b1111_1011; "true to false")]
    #[test_case(0xFF, true, 0xFF; "true to true")]
    fn it_can_set_timer_overflow(init: Byte, value: bool, expected: Byte) {
        let mut ie = InterruptEnable::default();
        ie.update(init);

        ie.set_timer_overflow(value);

        assert_eq!(Byte::from(&ie), expected);
    }

    #[test_case(0x00, false, 0x00; "false to false")]
    #[test_case(0x00, true, 0b0001_0000; "false to true")]
    #[test_case(0xFF, false, 0b1110_1111; "true to false")]
    #[test_case(0xFF, true, 0xFF; "true to true")]
    fn it_can_set_p10_p13_transition(init: Byte, value: bool, expected: Byte) {
        let mut ie = InterruptEnable::default();
        ie.update(init);

        ie.set_p10_p13_transition(value);

        assert_eq!(Byte::from(&ie), expected);
    }
}
