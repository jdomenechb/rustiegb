use crate::audio::channels::channel::{Channel, ChannelEvent};
use crate::audio::channels::default_channel::DefaultChannel;
use crate::audio::channels::sweep_channel::SweepChannel;
use crate::audio::registers::no_register::NoRegister;
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
use crate::audio::registers::{AudioRegister, WriteEffect};
use crate::bus::address::Address;
use crate::debug::Debuggable;
use crate::io::wave_pattern_ram::WavePatternRam;
use crate::memory::memory_sector::{ReadMemory, WriteMemory};
use crate::{Byte, Word};
use std::collections::BTreeMap;

pub struct Apu {
    channel_1: SweepChannel,
    channel_2: DefaultChannel<NoRegister, NRX1, NRX2, NRX3, NRX4>,
    channel_3: DefaultChannel<NR30, NR31, NR32, NRX3, NRX4>,
    channel_4: DefaultChannel<NoRegister, NR41, NRX2, NR43, NR44>,

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
        let length_step = self.div_apu.is_multiple_of(2);

        if length_step {
            let channel_event = self.channel_1.tick_length();
            self.process_channel_event(channel_event);

            let channel_event = self.channel_2.tick_length();
            self.process_channel_event(channel_event);

            let channel_event = self.channel_3.tick_length();
            self.process_channel_event(channel_event);

            let channel_event = self.channel_4.tick_length();
            self.process_channel_event(channel_event);
        }

        // Ticks every 128 Hz
        let sweep_step = matches!(self.div_apu, 2 | 6);

        if sweep_step {
            let channel_event = self.channel_1.tick_sweep();
            self.process_channel_event(channel_event);
        }

        // Ticks every 64 Hz
        let envelope_step = self.div_apu == 7;

        self.div_apu = (self.div_apu + 1) % 8;
    }

    fn clear_audio_registers(&mut self) {
        self.channel_1.clear();
        self.channel_2.clear();
        self.channel_3.clear();
        self.channel_4.clear();

        self.nr50 = 0;
        self.nr51 = 0;
    }

    fn process_channel_event(&mut self, channel_event: ChannelEvent) {
        match channel_event {
            ChannelEvent::ChannelEnabled(channel) => self.nr52.set_ro_channel_flag_active(channel),
            ChannelEvent::ChannelDisabled(channel) => {
                self.nr52.set_ro_channel_flag_inactive(channel)
            }
            ChannelEvent::None => (),
        }
    }
}

impl Default for Apu {
    fn default() -> Self {
        Self {
            channel_1: SweepChannel::new(),

            channel_2: DefaultChannel::new(
                2,
                NoRegister::default(),
                NRX1::new_nr21(),
                NRX2::new_nr22(),
                NRX3::default(),
                NRX4::default(),
                64,
            ),

            channel_3: DefaultChannel::new(
                3,
                NR30::default(),
                NR31::default(),
                NR32::default(),
                NRX3::default(),
                NRX4::default(),
                256,
            ),

            channel_4: DefaultChannel::new(
                4,
                NoRegister::default(),
                NR41::default(),
                NRX2::new_nr42(),
                NR43::default(),
                NR44::default(),
                64,
            ),

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
            Address::NR10_SOUND_1_SWEEP..=Address::NR14_SOUND_1_FR_HI => self
                .channel_1
                .read_byte(position - Address::NR10_SOUND_1_SWEEP),

            Address::NR20_SOUND_2_UNUSED..=Address::NR24_SOUND_2_FR_HI => self
                .channel_2
                .read_byte(position - Address::NR20_SOUND_2_UNUSED),

            Address::NR30_SOUND_3_ON_OFF..=Address::NR34_SOUND_3_FR_HI => self
                .channel_3
                .read_byte(position - Address::NR30_SOUND_3_ON_OFF),

            Address::NR40_SOUND_4_UNUSED..=Address::NR44_SOUND_4_CONTROL => self
                .channel_4
                .read_byte(position - Address::NR40_SOUND_4_UNUSED),

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
            let write_effect = self.nr52.write(value);

            match write_effect {
                WriteEffect::AudioOff => self.clear_audio_registers(),
                WriteEffect::AudioOn => self.div_apu = 0,
                WriteEffect::None => (),
                _ => unreachable!("WriteEffect not supported for NR52"),
            }

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

        let channel_event = match position {
            Address::NR10_SOUND_1_SWEEP..=Address::NR14_SOUND_1_FR_HI => {
                self.channel_1
                    .write_byte(position - Address::NR10_SOUND_1_SWEEP, value, &self.div_apu)
                    .0
            }

            Address::NR20_SOUND_2_UNUSED..=Address::NR24_SOUND_2_FR_HI => {
                self.channel_2
                    .write_byte(
                        position - Address::NR20_SOUND_2_UNUSED,
                        value,
                        &self.div_apu,
                    )
                    .0
            }

            Address::NR30_SOUND_3_ON_OFF..=Address::NR34_SOUND_3_FR_HI => {
                self.channel_3
                    .write_byte(
                        position - Address::NR30_SOUND_3_ON_OFF,
                        value,
                        &self.div_apu,
                    )
                    .0
            }

            Address::NR40_SOUND_4_UNUSED..=Address::NR44_SOUND_4_CONTROL => {
                self.channel_4
                    .write_byte(
                        position - Address::NR40_SOUND_4_UNUSED,
                        value,
                        &self.div_apu,
                    )
                    .0
            }

            Address::NR50 => {
                self.nr50 = value;
                ChannelEvent::None
            }
            Address::NR51 => {
                self.nr51 = value;
                ChannelEvent::None
            }

            _ => panic!("Write address {position:X} not supported for APU"),
        };

        self.process_channel_event(channel_event);
    }
}

impl Debuggable for Apu {
    fn get_debug_values(&self) -> BTreeMap<&str, String> {
        BTreeMap::from([
            // ("NR12", format!("{:X}", self.nr12.value())),
            // ("NR22", format!("{:X}", self.nr22.value())),
            // ("NR30", format!("{:X}", self.nr30.value())),
            // ("NR42", format!("{:X}", self.nr42.value())),
            // ("NR14", format!("{:X}", self.nr14.value())),
            // ("NR24", format!("{:X}", self.nr24.value())),
            // ("NR34", format!("{:X}", self.nr34.value())),
            // ("NR44", format!("{:X}", self.nr44.value())),
            // ("NR52", format!("{:X}", self.nr52.value())),
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
