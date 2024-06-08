use crate::io::UpdatableRegister;
use crate::Byte;

#[readonly::make]
pub struct NR10 {
    pub value: Byte,
}

impl UpdatableRegister for NR10 {
    fn update(&mut self, value: Byte) {
        self.value = value | 0b1000_0000;
    }
}

impl Default for NR10 {
    fn default() -> Self {
        Self { value: 0x80 }
    }
}
