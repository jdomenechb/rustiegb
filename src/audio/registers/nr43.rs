use crate::audio::registers::{AudioRegister, WriteEffect};
use crate::Byte;

/// Sweep
/// ```
/// 7 - RW - Clock swift
/// 6 - RW - Clock swift
/// 5 - RW - Clock swift
/// 4 - RW - Clock swift
/// 3 - RW - LFSR width
/// 2 - RW - Clock divider
/// 1 - RW - Clock divider
/// 0 - RW - Clock divider
/// ```
#[derive(Default)]
pub struct NR43 {
    value: Byte,
}

impl AudioRegister for NR43 {
    const READ_MASK: Byte = 0;
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
        let mut fixture = NR43::default();
        fixture.write(0xFF);
        assert_eq!(fixture.read(), 0xFF);

        fixture.write(0x00);
        assert_eq!(fixture.read(), 0x00);
    }
}
