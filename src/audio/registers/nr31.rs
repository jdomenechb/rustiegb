use crate::Byte;
use crate::audio::registers::{AudioRegister, InitialLengthRegister, WriteEffect};

/// Length timer - W
pub struct NR31 {
    value: Byte,
}

impl AudioRegister for NR31 {
    const READ_MASK: Byte = 0;
    const WRITE_MASK: Byte = 0;

    fn set_value(&mut self, value: Byte) -> WriteEffect {
        self.value = value;

        WriteEffect::None
    }

    fn value(&self) -> Byte {
        self.value
    }

    fn read(&self) -> Byte {
        0xFF
    }
}

impl Default for NR31 {
    fn default() -> Self {
        Self { value: 0xFF }
    }
}

impl InitialLengthRegister for NR31 {
    fn get_initial_length(&self) -> Byte {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_write_but_not_read() {
        let mut fixture = NR31::default();
        fixture.write(0xFF);
        assert_eq!(fixture.read(), 0xFF);

        fixture.write(0x00);
        assert_eq!(fixture.read(), 0xFF);
    }
}
