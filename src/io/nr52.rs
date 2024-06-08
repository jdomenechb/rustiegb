use crate::Byte;

#[derive(Clone)]
pub struct NR52 {
    value: Byte,
}

impl NR52 {
    pub fn is_on(&self) -> bool {
        self.value & 0b10000000 == 0b10000000
    }

    pub fn set_channel_active(&mut self, channel: u8) {
        self.value |= 0b1 << (channel - 1);
    }

    pub fn set_channel_inactive(&mut self, channel: u8) {
        self.value &= !(0b1 << (channel - 1));
    }
}

impl From<Byte> for NR52 {
    fn from(value: Byte) -> Self {
        Self { value }
    }
}

impl From<&NR52> for Byte {
    fn from(original: &NR52) -> Self {
        original.value
    }
}

impl Default for NR52 {
    fn default() -> Self {
        Self::from(0xf1)
    }
}
