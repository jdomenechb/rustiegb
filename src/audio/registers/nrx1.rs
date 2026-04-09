use crate::audio::registers::{AudioRegister, WriteEffect};
use crate::Byte;

/// Length timer & duty cycle
/// ```
/// 7 - RW - Wave duty
/// 6 - RW - Wave duty
/// 5 - W - Initial length timer
/// 4 - W - Initial length timer
/// 3 - W - Initial length timer
/// 2 - W - Initial length timer
/// 1 - W - Initial length timer
/// 0 - W - Initial length timer
/// ```
pub struct NRX1 {
    value: Byte,
}

impl NRX1 {
    pub fn new_nr11() -> Self {
        Self { value: 0xBF }
    }

    pub fn new_nr21() -> Self {
        Self { value: 0x3F }
    }
}

impl AudioRegister for NRX1 {
    const READ_MASK: Byte = 0b0011_1111;
    const WRITE_MASK: Byte = 0;

    fn set_value(&mut self, value: Byte) -> WriteEffect {
        self.value = value;

        WriteEffect::None
    }

    fn value(&self) -> Byte {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_read_and_write_the_value() {
        let mut fixture = NRX1::new_nr11();
        fixture.write(0xFF);
        assert_eq!(fixture.read(), 0xFF);

        fixture.write(0x00);
        assert_eq!(fixture.read(), 0x3F);
    }
}
