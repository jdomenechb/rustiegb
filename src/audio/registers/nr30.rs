use crate::audio::registers::{AudioRegister, DacAudioRegister};
use crate::Byte;

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

    fn set_value(&mut self, value: Byte) {
        self.value = value
    }

    fn value(&self) -> Byte {
        self.value
    }
}

impl Default for NR30 {
    fn default() -> Self {
        Self { value: 0x7F }
    }
}

impl DacAudioRegister for NR30 {
    fn is_going_to_turn_dac_off(&self, potential_value: &Byte) -> bool {
        potential_value & 0b10000000 == 0b00000000
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
