use crate::audio::registers::AudioRegister;
use crate::Byte;

/// Output level
/// ```
/// 7
/// 6 - RW - Output level
/// 5 - RW - Output level
/// 4
/// 3
/// 2
/// 1
/// 0
/// ```
pub struct NR32 {
    value: Byte,
}

impl AudioRegister for NR32 {
    const READ_MASK: Byte = 0b1001_1111;
    const WRITE_MASK: Byte = 0b1001_1111;

    fn set_value(&mut self, value: Byte) {
        self.value = value
    }

    fn value(&self) -> Byte {
        self.value
    }
}

impl Default for NR32 {
    fn default() -> Self {
        Self { value: 0x9F }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_read_and_write_the_value() {
        let mut fixture = NR32::default();
        fixture.write(0xFF);
        assert_eq!(fixture.read(), 0xFF);

        fixture.write(0x00);
        assert_eq!(fixture.read(), 0b1001_1111);
    }
}
