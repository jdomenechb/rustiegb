use crate::audio::channels::channel::{Channel, ChannelEvent, ChannelSignal};
use crate::audio::channels::default_channel::DefaultChannel;
use crate::audio::registers::WriteEffect;
use crate::audio::registers::nr30::NR30;
use crate::audio::registers::nr31::NR31;
use crate::audio::registers::nr32::NR32;
use crate::audio::registers::nrx3::NRX3;
use crate::audio::registers::nrx4::NRX4;
use crate::io::wave_pattern_ram::WavePatternRam;
use crate::memory::memory_sector::{ReadMemory, WriteMemory};
use crate::{Byte, Word};

/// Extra T-cycles before the first sample read after a trigger on DMG.
const DMG_TRIGGER_DELAY: i32 = 4;

/// T-cycles after a sample read at which a CPU read of the wave RAM still sees that sample.
const CPU_READ_DELAY: u32 = 2;

pub struct WaveChannel {
    channel: DefaultChannel<NR30, NR31, NR32, NRX3, NRX4>,
    wave_pattern_ram: WavePatternRam,

    frequency_timer: i32,
    sample_index: Word,
    cycles_since_read: u32,
    last_read_index: Word,
}

impl WaveChannel {
    pub fn new() -> Self {
        Self {
            channel: DefaultChannel::new(
                3,
                NR30::default(),
                NR31::default(),
                NR32::default(),
                NRX3::default(),
                NRX4::default(),
                256,
            ),
            wave_pattern_ram: WavePatternRam::default(),
            frequency_timer: 0,
            sample_index: 0,
            cycles_since_read: u32::MAX,
            last_read_index: 0,
        }
    }

    fn reset_frequency_timer(&mut self) {
        self.frequency_timer = (2048 - self.channel.read_frequency() as i32) * 2;
    }

    pub fn read_wave_pattern_byte(&self, position: Word, is_channel_on: bool) -> Byte {
        if !is_channel_on {
            return self.wave_pattern_ram.read_byte(position);
        }

        if self.cycles_since_read == CPU_READ_DELAY {
            return self.wave_pattern_ram.read_byte(self.last_read_index / 2);
        }

        0xFF
    }

    fn corrupt_wave_ram(&mut self) {
        let byte_index = self.sample_index / 2;

        if byte_index < 4 {
            let first = self.wave_pattern_ram.read_byte(byte_index);
            self.wave_pattern_ram.write_byte(0, first);
        } else {
            let block_start = byte_index & !0b11;

            for i in 0..4 {
                let value = self.wave_pattern_ram.read_byte(block_start + i);
                self.wave_pattern_ram.write_byte(i, value);
            }
        }
    }

    pub fn write_wave_pattern_byte(&mut self, position: Word, value: Byte, is_channel_on: bool) {
        if !is_channel_on {
            return self.wave_pattern_ram.write_byte(position, value);
        }

        if self.cycles_since_read == CPU_READ_DELAY {
            self.wave_pattern_ram
                .write_byte(self.sample_index / 2, value)
        }
    }
}

impl ReadMemory for WaveChannel {
    fn read_byte(&self, position: Word) -> Byte {
        self.channel.read_byte(position)
    }
}

impl Channel for WaveChannel {
    fn clear(&mut self) -> ChannelEvent {
        self.channel.clear()
    }

    fn write_byte(
        &mut self,
        position: Word,
        value: Byte,
        div_apu: &Byte,
        is_channel_on: bool,
    ) -> (ChannelEvent, WriteEffect) {
        let (channel_event, write_effect) =
            self.channel
                .write_byte(position, value, div_apu, is_channel_on);

        if write_effect == WriteEffect::Triggered {
            if is_channel_on && self.cycles_since_read == 0 {
                self.corrupt_wave_ram();
            }

            self.sample_index = 0;
            self.reset_frequency_timer();
            self.frequency_timer += DMG_TRIGGER_DELAY;
        }

        (channel_event, write_effect)
    }

    fn tick_length(&mut self) -> ChannelEvent {
        self.channel.tick_length()
    }

    fn single_step(&mut self) {
        self.frequency_timer -= 1;
        self.cycles_since_read = self.cycles_since_read.saturating_add(1);

        if self.frequency_timer == 0 {
            self.reset_frequency_timer();
            self.sample_index = (self.sample_index + 1) % 32;
            self.cycles_since_read = 0;
            self.last_read_index = self.sample_index;
        }
    }

    fn is_dac_on(&self) -> bool {
        self.channel.is_dac_on()
    }

    fn output_signal(&self) -> ChannelSignal {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAC_ON: Byte = 0x80;
    // Frequency 2047: the frequency timer is reloaded with 2 T-cycles
    const FAST_NR33: Byte = 0xFF;
    const FAST_TRIGGER_NR34: Byte = 0b1000_0111;
    // Frequency 2044: the frequency timer is reloaded with 8 T-cycles
    const SLOW_NR33: Byte = 0xFC;
    const SLOW_TRIGGER_NR34: Byte = 0b1000_0111;

    fn fill_wave_ram(fixture: &mut WaveChannel) {
        for i in 0..16 {
            fixture.write_wave_pattern_byte(i, (i as Byte) * 0x11, false);
        }
    }

    fn trigger(fixture: &mut WaveChannel, nr33: Byte, nr34: Byte, is_channel_on: bool) {
        fixture.write_byte(0, DAC_ON, &0, is_channel_on);
        fixture.write_byte(3, nr33, &0, is_channel_on);
        fixture.write_byte(4, nr34, &0, is_channel_on);
    }

    fn step(fixture: &mut WaveChannel, cycles: u32) {
        for _ in 0..cycles {
            fixture.single_step();
        }
    }

    #[test]
    fn it_reads_wave_ram_directly_when_channel_is_off() {
        let mut fixture = WaveChannel::new();
        fill_wave_ram(&mut fixture);

        assert_eq!(fixture.read_wave_pattern_byte(0x5, false), 0x55);
    }

    #[test]
    fn it_reads_0xff_from_wave_ram_when_channel_is_on_and_no_sample_was_just_read() {
        let mut fixture = WaveChannel::new();
        fill_wave_ram(&mut fixture);

        assert_eq!(fixture.read_wave_pattern_byte(0x5, true), 0xFF);
    }

    #[test]
    fn it_reads_the_current_sample_byte_only_2_cycles_after_a_sample_read() {
        let mut fixture = WaveChannel::new();
        fill_wave_ram(&mut fixture);
        trigger(&mut fixture, SLOW_NR33, SLOW_TRIGGER_NR34, false);

        // 8 cycles of timer + 4 cycles of DMG trigger delay
        step(&mut fixture, 12);
        assert_eq!(fixture.sample_index, 1);
        assert_eq!(fixture.read_wave_pattern_byte(0x5, true), 0xFF);

        step(&mut fixture, 1);
        assert_eq!(fixture.read_wave_pattern_byte(0x5, true), 0xFF);

        step(&mut fixture, 1);
        assert_eq!(fixture.read_wave_pattern_byte(0x5, true), 0x00);
    }

    #[test]
    fn it_writes_wave_ram_directly_when_channel_is_off() {
        let mut fixture = WaveChannel::new();

        fixture.write_wave_pattern_byte(0x5, 0xAB, false);

        assert_eq!(fixture.wave_pattern_ram.read_byte(0x5), 0xAB);
    }

    #[test]
    fn it_ignores_wave_ram_writes_when_channel_is_on_and_no_sample_was_just_read() {
        let mut fixture = WaveChannel::new();
        fill_wave_ram(&mut fixture);

        fixture.write_wave_pattern_byte(0x5, 0xAB, true);

        for i in 0..16 {
            assert_eq!(fixture.wave_pattern_ram.read_byte(i), (i as Byte) * 0x11);
        }
    }

    #[test]
    fn it_writes_to_the_current_sample_byte_only_2_cycles_after_a_sample_read() {
        let mut fixture = WaveChannel::new();
        fill_wave_ram(&mut fixture);
        trigger(&mut fixture, SLOW_NR33, SLOW_TRIGGER_NR34, false);

        // 8 cycles of timer + 4 cycles of DMG trigger delay
        step(&mut fixture, 12);
        assert_eq!(fixture.sample_index, 1);

        fixture.write_wave_pattern_byte(0xA, 0xF7, true);
        assert_eq!(fixture.wave_pattern_ram.read_byte(0), 0x00);

        step(&mut fixture, 1);
        fixture.write_wave_pattern_byte(0xA, 0xF7, true);
        assert_eq!(fixture.wave_pattern_ram.read_byte(0), 0x00);

        step(&mut fixture, 1);
        fixture.write_wave_pattern_byte(0xA, 0xF7, true);
        assert_eq!(fixture.wave_pattern_ram.read_byte(0), 0xF7);
        assert_eq!(fixture.wave_pattern_ram.read_byte(0xA), 0xAA);
    }

    #[test]
    fn it_delays_the_first_sample_read_after_trigger() {
        let mut fixture = WaveChannel::new();
        trigger(&mut fixture, FAST_NR33, FAST_TRIGGER_NR34, false);

        step(&mut fixture, 5);
        assert_eq!(fixture.sample_index, 0);

        step(&mut fixture, 1);
        assert_eq!(fixture.sample_index, 1);
    }

    #[test]
    fn it_wraps_the_sample_index_after_32_samples() {
        let mut fixture = WaveChannel::new();
        trigger(&mut fixture, FAST_NR33, FAST_TRIGGER_NR34, false);

        // First read at 6 cycles, then one every 2
        step(&mut fixture, 6 + 2 * 30);
        assert_eq!(fixture.sample_index, 31);

        step(&mut fixture, 2);
        assert_eq!(fixture.sample_index, 0);
    }

    #[test]
    fn it_resets_the_sample_index_when_triggered() {
        let mut fixture = WaveChannel::new();
        trigger(&mut fixture, FAST_NR33, FAST_TRIGGER_NR34, false);
        step(&mut fixture, 6 + 2 * 4);
        assert_eq!(fixture.sample_index, 5);

        fixture.write_byte(4, FAST_TRIGGER_NR34, &0, false);

        assert_eq!(fixture.sample_index, 0);
    }

    #[test]
    fn it_corrupts_the_first_byte_if_retriggered_while_reading_a_sample_in_the_first_block() {
        let mut fixture = WaveChannel::new();
        fill_wave_ram(&mut fixture);
        trigger(&mut fixture, FAST_NR33, FAST_TRIGGER_NR34, false);

        step(&mut fixture, 6 + 2 * 5);
        assert_eq!(fixture.sample_index, 6);

        fixture.write_byte(4, FAST_TRIGGER_NR34, &0, true);

        assert_eq!(fixture.wave_pattern_ram.read_byte(0), 0x33);
        assert_eq!(fixture.wave_pattern_ram.read_byte(1), 0x11);
    }

    #[test]
    fn it_corrupts_the_first_block_if_retriggered_while_reading_a_sample_in_another_block() {
        let mut fixture = WaveChannel::new();
        fill_wave_ram(&mut fixture);
        trigger(&mut fixture, FAST_NR33, FAST_TRIGGER_NR34, false);

        step(&mut fixture, 6 + 2 * 7);
        assert_eq!(fixture.sample_index, 8);

        fixture.write_byte(4, FAST_TRIGGER_NR34, &0, true);

        for i in 0..4 {
            assert_eq!(
                fixture.wave_pattern_ram.read_byte(i),
                (i as Byte + 4) * 0x11
            );
        }
        assert_eq!(fixture.wave_pattern_ram.read_byte(4), 0x44);
    }

    #[test]
    fn it_does_not_corrupt_wave_ram_if_retriggered_between_sample_reads() {
        let mut fixture = WaveChannel::new();
        fill_wave_ram(&mut fixture);
        trigger(&mut fixture, FAST_NR33, FAST_TRIGGER_NR34, false);

        step(&mut fixture, 6 + 2 * 7 + 1);

        fixture.write_byte(4, FAST_TRIGGER_NR34, &0, true);

        assert_eq!(fixture.wave_pattern_ram.read_byte(0), 0x00);
    }

    #[test]
    fn it_does_not_corrupt_wave_ram_if_retriggered_while_channel_is_off() {
        let mut fixture = WaveChannel::new();
        fill_wave_ram(&mut fixture);
        trigger(&mut fixture, FAST_NR33, FAST_TRIGGER_NR34, false);

        step(&mut fixture, 6 + 2 * 7);

        fixture.write_byte(4, FAST_TRIGGER_NR34, &0, false);

        assert_eq!(fixture.wave_pattern_ram.read_byte(0), 0x00);
    }
}
