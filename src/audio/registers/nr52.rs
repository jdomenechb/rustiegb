use crate::Byte;
use crate::audio::registers::{AudioRegister, WriteEffect};
use crate::utils::math::is_bit_set;

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
pub struct NR52 {
    value: Byte,
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
}

impl AudioRegister for NR52 {
    const READ_MASK: Byte = 0b0111_0000;
    const WRITE_MASK: Byte = 0;

    fn set_value(&mut self, value: Byte) -> WriteEffect {
        let was_on = self.is_on();
        self.value = (self.value & 0b0000_1111) | (value & 0b1000_0000) | 0b0111_0000;

        if !self.is_on() {
            self.value &= 0b0111_0000;
        }

        if !is_bit_set(&value, 7) {
            return WriteEffect::AudioOff;
        }

        if !was_on {
            return WriteEffect::AudioOn;
        }

        WriteEffect::None
    }

    fn value(&self) -> Byte {
        self.value
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
        fixture.set_ro_channel_flag_inactive(1);

        fixture.write(0xFF);
        assert_eq!(fixture.read(), 0b11110000);

        fixture.write(0x00);
        assert_eq!(fixture.read(), 0b01110000);

        fixture.write(0xFF);
        assert_eq!(fixture.read(), 0b11110000);
    }

    #[test]
    fn it_reports_all_channels_as_inactive_if_turned_off() {
        let mut fixture = NR52::default();
        fixture.set_ro_channel_flag_active(1);
        fixture.set_ro_channel_flag_active(2);
        fixture.set_ro_channel_flag_active(3);
        fixture.set_ro_channel_flag_active(4);

        fixture.write(0xFF);
        assert_eq!(fixture.read(), 0xFF);

        fixture.write(0x00);
        assert_eq!(fixture.read(), 0b01110000);
    }
}
