use crate::Byte;
use crate::audio::registers::{AudioRegister, DacRegister, WriteEffect};
use crate::utils::math::is_bit_set;

/// DAC enable
/// ```
/// 7 - RW - DAC On/Off
/// 6
/// 5
/// 4
/// 3
/// 2
/// 1
/// 0
/// ```
pub struct NR30 {
    value: Byte,
}

impl AudioRegister for NR30 {
    const READ_MASK: Byte = 0b0111_1111;
    const WRITE_MASK: Byte = 0b0111_1111;

    fn set_value(&mut self, value: Byte) -> WriteEffect {
        self.value = value;

        if !self.is_dac_enabled() {
            return WriteEffect::DacDisabled;
        }

        WriteEffect::DacEnabled
    }

    fn value(&self) -> Byte {
        self.value
    }
}

impl DacRegister for NR30 {
    fn is_dac_enabled(&self) -> bool {
        is_bit_set(&self.value, 7)
    }
}

impl Default for NR30 {
    fn default() -> Self {
        Self { value: 0x7F }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_read_and_write_the_value() {
        let mut fixture = NR30::default();
        fixture.write(0xFF);
        assert_eq!(fixture.read(), 0xFF);

        fixture.write(0x00);
        assert_eq!(fixture.read(), 0b0111_1111);
    }
}
