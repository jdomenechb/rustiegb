use crate::Byte;
use crate::audio::registers::{AudioRegister, InitialLengthRegister, WriteEffect};

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

        WriteEffect::NRX1Updated
    }

    fn value(&self) -> Byte {
        self.value
    }

    fn clear_because_of_audio_off(&mut self) {
        // Only the duty is cleared, the length is kept
        self.value &= 0b0011_1111;
    }
}

impl InitialLengthRegister for NRX1 {
    fn get_initial_length(&self) -> Byte {
        self.value & 0b0011_1111
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

    #[test]
    fn it_clears_only_the_duty_when_audio_is_off() {
        let mut fixture = NRX1::new_nr11();
        fixture.write(0xFF);

        fixture.clear_because_of_audio_off();

        assert_eq!(fixture.value(), 0b0011_1111);
        assert_eq!(fixture.get_initial_length(), 0b11_1111);
        assert_eq!(fixture.read(), 0x3F);
    }
}
