use crate::Byte;

#[derive(Clone)]
pub struct NR52 {
    nr52: Byte,
}

impl NR52 {
    pub fn is_on(&self) -> bool {
        self.nr52 & 0b10000000 == 0b10000000
    }

    pub fn set_channel_active(&mut self, channel: u8) {
        self.nr52 |= 0b1 << (channel - 1);
    }

    pub fn set_channel_inactive(&mut self, channel: u8) {
        self.nr52 &= !(0b1 << (channel - 1));
    }
}

impl From<Byte> for NR52 {
    fn from(value: Byte) -> Self {
        Self { nr52: value }
    }
}

impl From<&NR52> for Byte {
    fn from(original: &NR52) -> Self {
        original.nr52
    }
}

impl Default for NR52 {
    fn default() -> Self {
        Self::from(0xf1)
    }
}
