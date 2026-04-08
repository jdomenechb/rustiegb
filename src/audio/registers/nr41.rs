use crate::audio::registers::AudioRegister;
use crate::Byte;

/// Length timer
/// ```
/// 7
/// 6
/// 5 - W - Initial length timer
/// 4 - W - Initial length timer
/// 3 - W - Initial length timer
/// 2 - W - Initial length timer
/// 1 - W - Initial length timer
/// 0 - W - Initial length timer
/// ```
pub struct NR41 {
    value: Byte,
}

impl AudioRegister for NR41 {
    const READ_MASK: Byte = 0xFF;
    const WRITE_MASK: Byte = 0b1100_0000;

    fn set_value(&mut self, value: Byte) {
        self.value = value
    }

    fn value(&self) -> Byte {
        self.value
    }
}

impl Default for NR41 {
    fn default() -> Self {
        Self { value: 0xFF }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_write_but_not_read() {
        let mut fixture = NR41::default();
        fixture.write(0xFF);
        assert_eq!(fixture.read(), 0xFF);

        fixture.write(0x00);
        assert_eq!(fixture.read(), 0xFF);
    }
}
