use crate::audio::registers::AudioRegister;
use crate::Byte;

/// Period high & Control
/// ```
/// 7 - W - Trigger
/// 6 - RW - Length enable
/// 5
/// 4
/// 3
/// 2
/// 1
/// 0
/// ```
pub struct NR44 {
    value: Byte,
}

impl Default for NR44 {
    fn default() -> Self {
        Self { value: 0xBF }
    }
}

impl AudioRegister for NR44 {
    const READ_MASK: Byte = 0b1011_1111;
    const WRITE_MASK: Byte = 0b0011_1111;

    fn set_value(&mut self, value: Byte) {
        self.value = value
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
        let mut fixture = NR44::default();
        fixture.write(0xFF);
        assert_eq!(fixture.read(), 0xFF);

        fixture.write(0x00);
        assert_eq!(fixture.read(), 0b1011_1111);
    }
}
