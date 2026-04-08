use crate::audio::registers::AudioRegister;
use crate::Byte;

/// Period (low) - W
pub struct NRX3 {
    value: Byte,
}

impl AudioRegister for NRX3 {
    const READ_MASK: Byte = 0xFF;
    const WRITE_MASK: Byte = 0;

    fn set_value(&mut self, value: Byte) {
        self.value = value
    }

    fn value(&self) -> Byte {
        self.value
    }
}

impl Default for NRX3 {
    fn default() -> Self {
        Self { value: 0xFF }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_write_but_not_read() {
        let mut fixture = NRX3::default();
        fixture.write(0xFF);
        assert_eq!(fixture.read(), 0xFF);

        fixture.write(0x00);
        assert_eq!(fixture.read(), 0xFF);
    }
}
