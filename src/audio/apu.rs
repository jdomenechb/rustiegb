use crate::audio::registers::nr10::NR10;
use crate::audio::registers::nr30::NR30;
use crate::audio::registers::nr31::NR31;
use crate::audio::registers::nr32::NR32;
use crate::audio::registers::nr41::NR41;
use crate::audio::registers::nr43::NR43;
use crate::audio::registers::nr44::NR44;
use crate::audio::registers::nr52::NR52;
use crate::audio::registers::nrx1::NRX1;
use crate::audio::registers::nrx2::NRX2;
use crate::audio::registers::nrx3::NRX3;
use crate::audio::registers::nrx4::NRX4;
use crate::audio::registers::{AudioRegister, DacAudioRegister, TriggerableAudioRegister};
use crate::bus::address::Address;
use crate::debug::Debuggable;
use crate::io::wave_pattern_ram::WavePatternRam;
use crate::memory::memory_sector::{ReadMemory, WriteMemory};
use crate::{Byte, Word};
use std::collections::BTreeMap;

pub struct Apu {
    nr10: NR10,
    nr11: NRX1,
    nr12: NRX2,
    nr13: NRX3,
    nr14: NRX4,

    nr21: NRX1,
    nr22: NRX2,
    nr23: NRX3,
    nr24: NRX4,

    nr30: NR30,
    nr31: NR31,
    nr32: NR32,
    nr33: NRX3,
    nr34: NRX4,

    nr41: NR41,
    nr42: NRX2,
    nr43: NR43,
    nr44: NR44,

    nr50: Byte,
    nr51: Byte,
    nr52: NR52,

    wave_pattern_ram: WavePatternRam,

    div_apu: Byte,
}

impl Apu {
    /// Ticks every 512 Hz
    pub fn tick(&mut self) {
        // Ticks every 256 Hz
        let length_step = self.div_apu % 2;

        // Ticks every 128 Hz
        let sweep_step = matches!(self.div_apu, 2 | 6);

        // Ticks every 64 Hz
        let envelope_step = self.div_apu == 7;

        self.div_apu = (self.div_apu + 1) % 7;
    }

    fn write_nr52(&mut self, value: Byte) {
        if NR52::is_going_to_be_turned_off_by(value) {
            self.clear_audio_registers();
        }

        self.nr52.set_value(value);
    }

    fn clear_audio_registers(&mut self) {
        self.nr10.clear();
        self.nr11.clear();
        self.nr12.clear();
        self.nr13.clear();
        self.nr14.clear();
        self.nr21.clear();
        self.nr22.clear();
        self.nr23.clear();
        self.nr24.clear();
        self.nr30.clear();
        self.nr31.clear();
        self.nr32.clear();
        self.nr33.clear();
        self.nr34.clear();
        self.nr41.clear();
        self.nr42.clear();
        self.nr43.clear();
        self.nr44.clear();
        self.nr50 = 0;
        self.nr51 = 0;
    }

    fn write_nr12(&mut self, value: Byte) {
        if self.nr12.is_going_to_turn_dac_off(&value) {
            self.nr52.set_ro_channel_flag_inactive(1);
        }

        self.nr12.set_value(value);
    }

    fn write_nr14(&mut self, value: Byte) {
        if self.nr14.is_going_to_be_triggered_on_by(&value) {
            self.nr52.set_ro_channel_flag_active(1);
        }

        self.nr14.set_value(value);
    }

    fn write_nr22(&mut self, value: Byte) {
        if self.nr22.is_going_to_turn_dac_off(&value) {
            self.nr52.set_ro_channel_flag_inactive(2);
        }

        self.nr22.set_value(value);
    }

    fn write_nr24(&mut self, value: Byte) {
        if self.nr24.is_going_to_be_triggered_on_by(&value) {
            self.nr52.set_ro_channel_flag_active(2);
        }

        self.nr24.set_value(value);
    }

    fn write_nr30(&mut self, value: Byte) {
        if self.nr30.is_going_to_turn_dac_off(&value) {
            self.nr52.set_ro_channel_flag_inactive(3);
        }

        self.nr30.set_value(value);
    }

    fn write_nr34(&mut self, value: Byte) {
        if self.nr34.is_going_to_be_triggered_on_by(&value) {
            self.nr52.set_ro_channel_flag_active(3);
        }

        self.nr34.set_value(value);
    }

    fn write_nr42(&mut self, value: Byte) {
        if self.nr42.is_going_to_turn_dac_off(&value) {
            self.nr52.set_ro_channel_flag_inactive(4);
        }

        self.nr42.set_value(value);
    }

    fn write_nr44(&mut self, value: Byte) {
        if self.nr44.is_going_to_be_triggered_on_by(&value) {
            self.nr52.set_ro_channel_flag_active(4);
        }

        self.nr44.set_value(value);
    }
}

impl Default for Apu {
    fn default() -> Self {
        Self {
            nr10: NR10::default(),
            nr11: NRX1::new_nr11(),
            nr12: NRX2::new_nr12(),
            nr13: NRX3::default(),
            nr14: NRX4::default(),
            nr21: NRX1::new_nr21(),
            nr22: NRX2::new_nr22(),
            nr23: NRX3::default(),
            nr24: NRX4::default(),
            nr30: NR30::default(),
            nr31: NR31::default(),
            nr32: NR32::default(),
            nr33: NRX3::default(),
            nr34: NRX4::default(),
            nr41: NR41::default(),
            nr42: NRX2::new_nr42(),
            nr43: NR43::default(),
            nr44: NR44::default(),
            nr50: 0x77,
            nr51: 0xf3,
            nr52: NR52::default(),
            wave_pattern_ram: WavePatternRam::default(),
            div_apu: 0,
        }
    }
}

impl ReadMemory for Apu {
    fn read_byte(&self, position: Word) -> Byte {
        match position {
            Address::NR10_SOUND_1_SWEEP => self.nr10.read(),
            Address::NR11_SOUND_1_WAVE_PATTERN_DUTY => self.nr11.read(),
            Address::NR12_SOUND_1_ENVELOPE => self.nr12.read(),
            Address::NR13_SOUND_1_FR_LO => self.nr13.read(),
            Address::NR14_SOUND_1_FR_HI => self.nr14.read(),

            Address::NR20_SOUND_2_UNUSED => 0xFF,
            Address::NR21_SOUND_2_WAVE_PATTERN_DUTY => self.nr21.read(),
            Address::NR22_SOUND_2_ENVELOPE => self.nr22.read(),
            Address::NR23_SOUND_2_FR_LO => self.nr23.read(),
            Address::NR24_SOUND_2_FR_HI => self.nr24.read(),

            Address::NR30_SOUND_3_ON_OFF => self.nr30.read(),
            Address::NR31_SOUND_3_LENGTH => self.nr31.read(),
            Address::NR32_SOUND_3_OUTPUT_LEVEL => self.nr32.read(),
            Address::NR33_SOUND_3_FR_LO => self.nr33.read(),
            Address::NR34_SOUND_3_FR_HI => self.nr34.read(),

            Address::NR40_SOUND_4_UNUSED => 0xFF,
            Address::NR41_SOUND_4_LENGTH => self.nr41.read(),
            Address::NR42_SOUND_4_ENVELOPE => self.nr42.read(),
            Address::NR43_SOUND_4_FR_RANDOMNESS => self.nr43.read(),
            Address::NR44_SOUND_4_CONTROL => self.nr44.read(),

            Address::NR50 => self.nr50,
            Address::NR51 => self.nr51,
            Address::NR52_SOUND => self.nr52.read(),

            Address::WAVE_PATTERN_START..=Address::WAVE_PATTERN_END => self
                .wave_pattern_ram
                .read_byte(position - Address::WAVE_PATTERN_START),
            _ => {
                println!("Read address {position:X} not supported for APU");
                0xFF
            }
        }
    }
}

impl WriteMemory for Apu {
    fn write_byte(&mut self, position: Word, value: Byte) {
        if position == Address::NR52_SOUND {
            self.write_nr52(value);
            return;
        }

        if (Address::WAVE_PATTERN_START..=Address::WAVE_PATTERN_END).contains(&position) {
            self.wave_pattern_ram
                .write_byte(position - Address::WAVE_PATTERN_START, value);

            return;
        }

        if !self.nr52.is_on() {
            return;
        }

        match position {
            Address::NR10_SOUND_1_SWEEP => self.nr10.write(value),
            Address::NR11_SOUND_1_WAVE_PATTERN_DUTY => self.nr11.write(value),
            Address::NR12_SOUND_1_ENVELOPE => self.write_nr12(value),
            Address::NR13_SOUND_1_FR_LO => self.nr13.write(value),
            Address::NR14_SOUND_1_FR_HI => self.write_nr14(value),

            Address::NR20_SOUND_2_UNUSED => {
                // Ignored, not used
            }
            Address::NR21_SOUND_2_WAVE_PATTERN_DUTY => self.nr21.write(value),
            Address::NR22_SOUND_2_ENVELOPE => self.write_nr22(value),
            Address::NR23_SOUND_2_FR_LO => self.nr23.write(value),
            Address::NR24_SOUND_2_FR_HI => self.write_nr24(value),

            Address::NR30_SOUND_3_ON_OFF => self.write_nr30(value),
            Address::NR31_SOUND_3_LENGTH => self.nr31.write(value),
            Address::NR32_SOUND_3_OUTPUT_LEVEL => self.nr32.write(value),
            Address::NR33_SOUND_3_FR_LO => self.nr33.write(value),
            Address::NR34_SOUND_3_FR_HI => self.write_nr34(value),

            Address::NR40_SOUND_4_UNUSED => {
                // Ignored, not used
            }
            Address::NR41_SOUND_4_LENGTH => self.nr41.write(value),
            Address::NR42_SOUND_4_ENVELOPE => self.write_nr42(value),
            Address::NR43_SOUND_4_FR_RANDOMNESS => self.nr43.write(value),
            Address::NR44_SOUND_4_CONTROL => self.write_nr44(value),

            Address::NR50 => self.nr50 = value,
            Address::NR51 => self.nr51 = value,

            _ => panic!("Write address {position:X} not supported for APU"),
        }
    }
}

impl Debuggable for Apu {
    fn get_debug_values(&self) -> BTreeMap<&str, String> {
        BTreeMap::from([
            ("NR12", format!("{:X}", self.nr12.value())),
            ("NR22", format!("{:X}", self.nr22.value())),
            ("NR30", format!("{:X}", self.nr30.value())),
            ("NR42", format!("{:X}", self.nr42.value())),
            ("NR14", format!("{:X}", self.nr14.value())),
            ("NR24", format!("{:X}", self.nr24.value())),
            ("NR34", format!("{:X}", self.nr34.value())),
            ("NR44", format!("{:X}", self.nr44.value())),
            ("NR52", format!("{:X}", self.nr52.value())),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_basic_audio_registers_are_reset(apu: &mut Apu) {
        let items = vec![
            // NR1X
            (Address::NR10_SOUND_1_SWEEP, 0x80),
            (Address::NR11_SOUND_1_WAVE_PATTERN_DUTY, 0x3F),
            (Address::NR12_SOUND_1_ENVELOPE, 0x00),
            (Address::NR13_SOUND_1_FR_LO, 0xFF),
            (Address::NR14_SOUND_1_FR_HI, 0xBF),
            // NR2X
            (Address::NR20_SOUND_2_UNUSED, 0xFF),
            (Address::NR21_SOUND_2_WAVE_PATTERN_DUTY, 0x3F),
            (Address::NR22_SOUND_2_ENVELOPE, 0x00),
            (Address::NR23_SOUND_2_FR_LO, 0xFF),
            (Address::NR24_SOUND_2_FR_HI, 0xBF),
            // NR3X
            (Address::NR30_SOUND_3_ON_OFF, 0x7F),
            (Address::NR31_SOUND_3_LENGTH, 0xFF),
            (Address::NR32_SOUND_3_OUTPUT_LEVEL, 0x9F),
            (Address::NR33_SOUND_3_FR_LO, 0xFF),
            (Address::NR34_SOUND_3_FR_HI, 0xBF),
            // NR4X
            (Address::NR40_SOUND_4_UNUSED, 0xFF),
            (Address::NR41_SOUND_4_LENGTH, 0xFF),
            (Address::NR42_SOUND_4_ENVELOPE, 0x00),
            (Address::NR43_SOUND_4_FR_RANDOMNESS, 0x00),
            (Address::NR44_SOUND_4_CONTROL, 0xBF),
            // NR5X
            (Address::NR50, 0x00),
            (Address::NR51, 0x00),
            // NR52 Skipped as it is special
        ];

        for item in items {
            assert_eq!(
                apu.read_byte(item.0),
                item.1,
                "Wrong data when writing register {:X}",
                item.0
            );
        }
    }

    #[test]
    fn test_correct_data_when_writing_audio_registers() {
        let mut apu = Apu::default();

        for position in Address::NR10_SOUND_1_SWEEP..=Address::NR51 {
            apu.write_byte(position, 0xFF);
            apu.write_byte(position, 0);
        }

        check_basic_audio_registers_are_reset(&mut apu);
    }

    #[test]
    fn test_when_sound_is_turned_off_all_audio_registers_are_reset() {
        let mut apu = Apu::default();

        for position in Address::NR10_SOUND_1_SWEEP..=Address::NR51 {
            apu.write_byte(position, 0xFF);
        }

        apu.write_byte(Address::NR52_SOUND, 0);
        apu.write_byte(Address::NR52_SOUND, 0b10000000);

        check_basic_audio_registers_are_reset(&mut apu);
    }

    #[test]
    fn test_when_sound_is_turned_off_audio_registers_ignore_writes() {
        let mut apu = Apu::default();

        for position in Address::NR10_SOUND_1_SWEEP..=Address::NR51 {
            apu.write_byte(position, 0x00);
        }

        apu.write_byte(Address::NR52_SOUND, 0);

        for position in Address::NR10_SOUND_1_SWEEP..=Address::NR51 {
            apu.write_byte(position, 0xFF);
        }

        check_basic_audio_registers_are_reset(&mut apu);
    }

    #[test]
    fn test_when_sound_is_turned_off_wave_pattern_register_is_writable() {
        let mut apu = Apu::default();

        apu.write_byte(Address::NR52_SOUND, 0);

        for position in Address::WAVE_PATTERN_START..=Address::WAVE_PATTERN_END {
            apu.write_byte(position, 0xFF);
            assert_eq!(0xFF, apu.read_byte(position));
        }
    }

    #[test]
    fn test_correct_data_when_writing_wave_registers() {
        let mut apu = Apu::default();

        // WAVE
        for position in Address::WAVE_PATTERN_START..=Address::WAVE_PATTERN_END {
            apu.write_byte(position, 0xFF);
            apu.write_byte(position, 0);

            assert_eq!(
                apu.read_byte(position),
                0,
                "Wrong data when writing register {:X}",
                position
            );
        }
    }

    #[test]
    fn test_when_channel_is_triggered_nr52_channel_flag_is_set() {
        let mut apu = Apu::default();

        for channel in 1..=4 {
            let address = get_trigger_address(channel);

            apu.write_byte(address, 0b1000_0000);

            let flag = (apu.read_byte(Address::NR52_SOUND) >> (channel - 1) & 0b1) == 1;
            assert!(flag, "Channel {} should be active", channel);
        }
    }

    #[test]
    fn test_when_dac_is_turned_off_channel_flag_is_set_off() {
        let mut apu = Apu::default();

        for channel in 1..=4 {
            let trigger_address = get_trigger_address(channel);
            apu.write_byte(trigger_address, 0b1000_0000);

            let dac_address = get_dac_address(channel);

            apu.write_byte(dac_address, 0);

            let flag = (apu.read_byte(Address::NR52_SOUND) >> (channel - 1) & 0b1) == 0;
            assert!(flag, "Channel {} should not be active", channel);
        }
    }

    fn get_trigger_address(channel: u8) -> Word {
        match channel {
            1 => Address::NR14_SOUND_1_FR_HI,
            2 => Address::NR24_SOUND_2_FR_HI,
            3 => Address::NR34_SOUND_3_FR_HI,
            4 => Address::NR44_SOUND_4_CONTROL,
            _ => unreachable!(),
        }
    }

    fn get_dac_address(channel: u8) -> Word {
        match channel {
            1 => Address::NR12_SOUND_1_ENVELOPE,
            2 => Address::NR22_SOUND_2_ENVELOPE,
            3 => Address::NR30_SOUND_3_ON_OFF,
            4 => Address::NR42_SOUND_4_ENVELOPE,
            _ => unreachable!(),
        }
    }

    // TODO: Implement DIV-APU
}
