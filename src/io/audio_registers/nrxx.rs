use crate::io::ResettableRegister;
use crate::Byte;

pub struct NRxx {
    pub value: Byte,
}

impl NRxx {
    pub fn new(default: Byte) -> Self {
        Self { value: default }
    }
}

impl ResettableRegister for NRxx {
    fn reset(&mut self) {
        self.value = 0x00;
    }
}
