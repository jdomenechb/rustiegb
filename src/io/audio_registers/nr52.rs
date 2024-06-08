use crate::Byte;

#[derive(Clone)]
#[readonly::make]
pub struct NR52 {
    pub value: Byte,
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

    pub fn update(&mut self, value: Byte) {
        self.value = (value & 0b10000000) | 0b01110000
    }
}

impl Default for NR52 {
    fn default() -> Self {
        Self { value: 0xf1 }
    }
}
