use crate::bus::address::Address;
use crate::debug::Debuggable;
use crate::io::audio_registers::nr52::NR52;
use crate::io::audio_registers::nrxx::{NRxx, NRxxProperties};
use crate::io::audio_registers::{AudioRegWritten, AudioRegisters};
use crate::memory::memory_sector::{ReadMemory, WriteMemory};
use crate::{Byte, Word};
use std::collections::BTreeMap;

pub struct Apu {
    nr10: NRxx,
    nr11: NRxx,
    nr12: NRxx,
    nr13: NRxx,
    nr14: NRxx,

    nr21: NRxx,
    nr22: NRxx,
    nr23: NRxx,
    nr24: NRxx,

    nr30: NRxx,
    nr31: NRxx,
    nr32: NRxx,
    nr33: NRxx,
    nr34: NRxx,

    nr41: NRxx,
    nr42: NRxx,
    nr43: NRxx,
    nr44: NRxx,

    nr50: Byte,
    nr51: Byte,
    pub nr52: NR52,

    // TO DELETE

    // --- OTHER - Out of scope of register
    // Audio
    pub audio_1_reg_written: AudioRegWritten,
    pub audio_2_reg_written: AudioRegWritten,
    pub audio_3_reg_written: AudioRegWritten,
    pub audio_4_reg_written: AudioRegWritten,
}

impl Apu {
    pub fn step(&mut self) {}

    pub fn audio_reg_have_been_written(
        &mut self,
    ) -> (
        AudioRegWritten,
        AudioRegWritten,
        AudioRegWritten,
        AudioRegWritten,
    ) {
        let to_return = (
            self.audio_1_reg_written.clone(),
            self.audio_2_reg_written.clone(),
            self.audio_3_reg_written.clone(),
            self.audio_4_reg_written.clone(),
        );

        self.audio_1_reg_written = AudioRegWritten::default();
        self.audio_2_reg_written = AudioRegWritten::default();
        self.audio_3_reg_written = AudioRegWritten::default();
        self.audio_4_reg_written = AudioRegWritten::default();

        to_return
    }

    pub fn update_audio_1_frequency(&mut self, frequency: Word) {
        self.nr13.update_low_frequency(frequency);
        self.nr14.update_high_frequency(frequency)
    }

    pub fn read_audio_registers(&self, channel: u8) -> AudioRegisters {
        match channel {
            1 => AudioRegisters::new(
                self.nr14.value,
                self.nr13.value,
                self.nr12.value,
                self.nr11.value,
                Some(self.nr10.value),
            ),
            2 => AudioRegisters::new(
                self.nr24.value,
                self.nr23.value,
                self.nr22.value,
                self.nr21.value,
                None,
            ),
            3 => AudioRegisters::new(
                self.nr34.value,
                self.nr33.value,
                self.nr32.value,
                self.nr31.value,
                Some(self.nr30.value),
            ),
            4 => AudioRegisters::new(
                self.nr44.value,
                self.nr43.value,
                self.nr42.value,
                self.nr41.value,
                None,
            ),
            _ => panic!("Invalid channel provided"),
        }
    }

    fn should_channel_be_turned_on(&self, channel: u8) -> bool {
        let (nrx4, nrx2) = match channel {
            1 => (self.nr14.value, self.nr12.value),
            2 => (self.nr24.value, self.nr22.value),
            3 => (self.nr34.value, self.nr32.value),
            4 => (self.nr44.value, self.nr42.value),
            _ => panic!("Invalid channel given"),
        };

        (nrx4 & 0b1000_0000) == 0b1000_0000 && (nrx2 & 0b1111_1000) != 0b0000_0000
    }
}

impl Default for Apu {
    fn default() -> Self {
        Self {
            nr10: NRxx::new_from_properties(
                0x80,
                NRxxProperties::default().with_used_bits(0b0111_1111),
            ),
            nr11: NRxx::new_from_properties(
                0xBF,
                NRxxProperties::default().with_only_writable_bits(0b0011_1111),
            ),
            nr12: NRxx::new(0xF3),
            nr13: NRxx::new_from_properties(
                0xFF,
                NRxxProperties::default().with_only_writable_bits(0xFF),
            ),
            nr14: NRxx::new_from_properties(
                0xBF,
                NRxxProperties::default()
                    .with_used_bits(0b1100_0111)
                    .with_only_writable_bits(0b1011_1111)
                    .with_avoid_reset_on_bits(0b1000_0000),
            ),

            nr21: NRxx::new_from_properties(
                0x3F,
                NRxxProperties::default().with_only_writable_bits(0x3F),
            ),
            nr22: NRxx::new(0x00),
            nr23: NRxx::new_from_properties(
                0xFF,
                NRxxProperties::default().with_only_writable_bits(0xFF),
            ),
            nr24: NRxx::new_from_properties(
                0xBF,
                NRxxProperties::default()
                    .with_used_bits(0b1100_0111)
                    .with_only_writable_bits(0xBF)
                    .with_avoid_reset_on_bits(0b1000_0000),
            ),

            nr30: NRxx::new_from_properties(
                0x7F,
                NRxxProperties::default().with_used_bits(0b1000_0000),
            ),
            nr31: NRxx::new_from_properties(
                0xFF,
                NRxxProperties::default().with_only_writable_bits(0xFF),
            ),
            nr32: NRxx::new_from_properties(
                0x9F,
                NRxxProperties::default().with_used_bits(0b0110_0000),
            ),
            nr33: NRxx::new_from_properties(
                0xFF,
                NRxxProperties::default().with_only_writable_bits(0xFF),
            ),
            nr34: NRxx::new_from_properties(
                0xBF,
                NRxxProperties::default()
                    .with_used_bits(0b1100_0111)
                    .with_only_writable_bits(0xBF)
                    .with_avoid_reset_on_bits(0b1000_0000),
            ),

            nr41: NRxx::new_from_properties(
                0xFF,
                NRxxProperties::default()
                    .with_used_bits(0b0011_1111)
                    .with_only_writable_bits(0xFF),
            ),
            nr42: NRxx::new(0x00),
            nr43: NRxx::new(0x00),
            nr44: NRxx::new_from_properties(
                0xBF,
                NRxxProperties::default()
                    .with_used_bits(0b1100_0000)
                    .with_only_writable_bits(0b1000_0000)
                    .with_avoid_reset_on_bits(0b1000_0000),
            ),

            nr50: 0x77,
            nr51: 0xf3,
            nr52: NR52::default(),
            audio_1_reg_written: AudioRegWritten::default(),
            audio_2_reg_written: AudioRegWritten::default(),
            audio_3_reg_written: AudioRegWritten::default(),
            audio_4_reg_written: AudioRegWritten::default(),
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
            Address::NR52_SOUND => self.nr52.value,

            _ => {
                println!("Read address {:X} not supported for APU", position);
                0xFF
            }
        }
    }
}

impl WriteMemory for Apu {
    fn write_byte(&mut self, position: Word, value: Byte) {
        match position {
            Address::NR10_SOUND_1_SWEEP => {
                if self.nr52.is_on() {
                    self.nr10.update(value);
                    self.audio_1_reg_written.sweep_or_wave_onoff = true;
                }
            }
            Address::NR11_SOUND_1_WAVE_PATTERN_DUTY => {
                if self.nr52.is_on() {
                    self.nr11.update(value);
                    self.audio_1_reg_written.length = true;
                }
            }
            Address::NR12_SOUND_1_ENVELOPE => {
                if self.nr52.is_on() {
                    self.nr12.update(value);
                    self.audio_1_reg_written.envelope_or_wave_out_lvl = true;
                }
            }
            Address::NR13_SOUND_1_FR_LO => {
                if self.nr52.is_on() {
                    self.nr13.update(value);
                    self.audio_1_reg_written.frequency_or_poly_counter = true;
                }
            }
            Address::NR14_SOUND_1_FR_HI => {
                if !self.nr52.is_on() {
                    return;
                }

                self.audio_1_reg_written.control = true;

                self.nr14.update(value);

                if self.should_channel_be_turned_on(1) {
                    self.nr52.set_ro_channel_flag_active(1);
                }
            }
            Address::NR21_SOUND_2_WAVE_PATTERN_DUTY => {
                if self.nr52.is_on() {
                    self.nr21.update(value);
                    self.audio_2_reg_written.length = true;
                }
            }
            Address::NR22_SOUND_2_ENVELOPE => {
                if self.nr52.is_on() {
                    self.nr22.update(value);
                    self.audio_2_reg_written.envelope_or_wave_out_lvl = true;
                }
            }
            Address::NR23_SOUND_2_FR_LO => {
                if self.nr52.is_on() {
                    self.nr23.update(value);
                    self.audio_2_reg_written.frequency_or_poly_counter = true;
                }
            }
            Address::NR24_SOUND_2_FR_HI => {
                if !self.nr52.is_on() {
                    return;
                }

                self.audio_2_reg_written.control = true;

                if value & 0b10000000 == 0b10000000 {
                    self.nr52.set_ro_channel_flag_active(2);
                }

                self.nr24.update(value);
            }
            Address::NR30_SOUND_3_ON_OFF => {
                if self.nr52.is_on() {
                    self.nr30.update(value);
                    self.audio_3_reg_written.sweep_or_wave_onoff = true;
                }
            }
            Address::NR31_SOUND_3_LENGTH => {
                if self.nr52.is_on() {
                    self.nr31.update(value);
                    self.audio_3_reg_written.length = true;
                }
            }
            Address::NR32_SOUND_3_OUTPUT_LEVEL => {
                if self.nr52.is_on() {
                    self.nr32.update(value);
                }
            }
            Address::NR33_SOUND_3_FR_LO => {
                if self.nr52.is_on() {
                    self.nr33.update(value);
                    self.audio_3_reg_written.frequency_or_poly_counter = true;
                }
            }
            Address::NR34_SOUND_3_FR_HI => {
                if !self.nr52.is_on() {
                    return;
                }

                self.audio_3_reg_written.control = true;

                if value & 0b10000000 == 0b10000000 {
                    self.nr52.set_ro_channel_flag_active(3);
                }

                self.nr34.update(value);
            }
            Address::NR41_SOUND_4_LENGTH => {
                if self.nr52.is_on() {
                    self.nr41.update(value);
                    self.audio_4_reg_written.length = true;
                }
            }
            Address::NR42_SOUND_4_ENVELOPE => {
                if self.nr52.is_on() {
                    self.nr42.update(value);
                    self.audio_4_reg_written.envelope_or_wave_out_lvl = true;
                }
            }
            Address::NR43_SOUND_4_FR_RANDOMNESS => {
                if self.nr52.is_on() {
                    self.nr43.update(value);
                    self.audio_4_reg_written.frequency_or_poly_counter = true;
                }
            }
            Address::NR44_SOUND_4_CONTROL => {
                if !self.nr52.is_on() {
                    return;
                }

                self.audio_4_reg_written.control = true;

                if value & 0b10000000 == 0b10000000 {
                    self.nr52.set_ro_channel_flag_active(4);
                }

                self.nr44.update(value);
            }
            Address::NR50 => {
                if self.nr52.is_on() {
                    self.nr50 = value;
                }
            }
            Address::NR51 => {
                if self.nr52.is_on() {
                    self.nr51 = value;
                }
            }
            Address::NR52_SOUND => {
                self.nr52.update(value);

                if self.nr52.is_on() {
                    self.nr50 = 0;
                    self.nr51 = 0;
                } else {
                    self.nr10.reset();
                    self.nr11.reset();
                    self.nr12.reset();
                    self.nr13.reset();
                    self.nr14.reset();

                    // NR20 is not used
                    self.nr21.reset();
                    self.nr22.reset();
                    self.nr23.reset();
                    self.nr24.reset();

                    self.nr30.reset();
                    self.nr31.reset();
                    self.nr32.reset();
                    self.nr33.reset();
                    self.nr34.reset();

                    self.nr41.reset();
                    self.nr42.reset();
                    self.nr43.reset();
                    self.nr44.reset();

                    self.nr52.set_ro_channel_flag_inactive(1);
                    self.nr52.set_ro_channel_flag_inactive(2);
                    self.nr52.set_ro_channel_flag_inactive(3);
                    self.nr52.set_ro_channel_flag_inactive(4);
                }
            }
            Address::NR20_SOUND_2_UNUSED => {
                // Ignored, not used
            }
            Address::NR40_SOUND_4_UNUSED => {
                // Ignored, not used
            }
            _ => panic!("Write address {:X} not supported for APU", position),
        }
    }
}

impl Debuggable for Apu {
    fn get_debug_values(&self) -> BTreeMap<&str, String> {
        BTreeMap::from([
            ("NR12", format!("{:X}", self.nr12.value)),
            ("NR13", format!("{:X}", self.nr13.value)),
            ("NR14", format!("{:X}", self.nr14.value)),
            ("NR24", format!("{:X}", self.nr24.value)),
            ("NR34", format!("{:X}", self.nr34.value)),
            ("NR44", format!("{:X}", self.nr44.value)),
            ("NR52", format!("{:X}", self.nr52.value)),
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

    // TODO: Implement DIV-APU
}
