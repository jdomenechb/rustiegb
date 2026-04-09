use crate::Byte;
use crate::audio::registers::{AudioRegister, WriteEffect};

pub struct NoRegister {
    value: Byte,
}

impl AudioRegister for NoRegister {
    const READ_MASK: Byte = 0xFF;
    const WRITE_MASK: Byte = 0;

    fn set_value(&mut self, value: Byte) -> WriteEffect {
        self.value = value;

        WriteEffect::None
    }

    fn value(&self) -> Byte {
        self.value
    }
}

impl Default for NoRegister {
    fn default() -> Self {
        Self { value: 0xFF }
    }
}
