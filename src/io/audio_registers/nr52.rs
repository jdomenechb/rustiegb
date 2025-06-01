use crate::Byte;

/// Audio master control
/// ```
/// 7 - RW - Audio on/off
/// 6 - N/A
/// 5 - N/A
/// 4 - N/A
/// 3 - RO - CH4 on?
/// 2 - RO - CH3 on?
/// 1 - RO - CH2 on?
/// 0 - RO - CH1 on?
/// ```
#[derive(Clone)]
#[readonly::make]
pub struct NR52 {
    pub value: Byte,
}

impl NR52 {
    pub fn is_on(&self) -> bool {
        self.value & 0b10000000 == 0b10000000
    }

    pub fn set_ro_channel_flag_active(&mut self, channel: u8) {
        self.value |= 0b1 << (channel - 1);
    }

    pub fn set_ro_channel_flag_inactive(&mut self, channel: u8) {
        self.value &= !(0b1 << (channel - 1));
    }

    pub fn update(&mut self, value: Byte) {
        self.value = (self.value & 0b0000_1111) | (value & 0b1000_0000) | 0b0111_0000
    }
}

impl Default for NR52 {
    fn default() -> Self {
        Self {
            value: 0b11110001, // 0xF1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_update_only_the_write_flags() {
        let mut fixture = NR52::default();

        fixture.update(0xFF);
        assert_eq!(fixture.value, 0b11110001);

        fixture.update(0x00);
        assert_eq!(fixture.value, 0b01110001);

        fixture.update(0xFF);
        assert_eq!(fixture.value, 0b11110001);
    }
}
