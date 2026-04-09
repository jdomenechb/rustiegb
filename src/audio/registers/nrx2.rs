use crate::audio::registers::{AudioRegister, WriteEffect};
use crate::Byte;

/// Volume & envelope
/// ```
/// 7 - RW - Initial volume
/// 6 - RW - Initial volume
/// 5 - RW - Initial volume
/// 4 - RW - Initial volume
/// 3 - RW - Envelope dir
/// 2 - RW - Sweep pace
/// 1 - RW - Sweep pace
/// 0 - RW - Sweep pace
/// ```
pub struct NRX2 {
    value: Byte,
}

impl NRX2 {
    pub fn new_nr12() -> Self {
        Self { value: 0xF3 }
    }

    pub fn new_nr22() -> Self {
        Self { value: 0x00 }
    }

    pub fn new_nr42() -> Self {
        Self { value: 0x00 }
    }
}

impl AudioRegister for NRX2 {
    const READ_MASK: Byte = 0;
    const WRITE_MASK: Byte = 0;

    fn set_value(&mut self, value: Byte) -> WriteEffect {
        self.value = value;

        if value & 0b1111_1000 == 0 {
            return WriteEffect::DacDisabled;
        }

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
        let mut fixture = NRX2::new_nr12();
        fixture.write(0xFF);
        assert_eq!(fixture.read(), 0xFF);

        fixture.write(0x00);
        assert_eq!(fixture.read(), 0x00);
    }
}
