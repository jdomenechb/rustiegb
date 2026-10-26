use crate::audio::channels::channel::{Channel, ChannelEvent};
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
const DMG_TRIGGER_DELAY: i32 = 6;

pub struct WaveChannel {
    channel: DefaultChannel<NR30, NR31, NR32, NRX3, NRX4>,
    wave_pattern_ram: WavePatternRam,

    frequency_timer: i32,
    sample_index: Word,
    just_read: bool,
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
            just_read: false,
        }
    }

    fn reset_frequency_timer(&mut self) {
        self.frequency_timer = (2048 - self.channel.read_frequency() as i32) * 2;
    }

    pub fn read_wave_pattern_byte(&self, position: Word, is_channel_on: bool) -> Byte {
        if !is_channel_on {
            return self.wave_pattern_ram.read_byte(position);
        }

        if self.just_read {
            return self.wave_pattern_ram.read_byte(self.sample_index / 2);
        }

        0xFF
    }

    pub fn write_wave_pattern_byte(&mut self, position: Word, value: Byte) {
        self.wave_pattern_ram.write_byte(position, value)
    }
}

impl ReadMemory for WaveChannel {
    fn read_byte(&self, position: Word) -> Byte {
        self.channel.read_byte(position)
    }
}

impl Channel for WaveChannel {
    fn clear(&mut self) {
        self.channel.clear()
    }

    fn clear_because_of_audio_off(&mut self) {
        self.channel.clear_because_of_audio_off()
    }

    fn write_byte(
        &mut self,
        position: Word,
        value: Byte,
        div_apu: &Byte,
    ) -> (ChannelEvent, WriteEffect) {
        let (channel_event, write_effect) = self.channel.write_byte(position, value, div_apu);

        if write_effect == WriteEffect::Triggered {
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
        self.just_read = false;

        if self.frequency_timer == 0 {
            self.reset_frequency_timer();
            self.sample_index = (self.sample_index + 1) % 32;
            self.just_read = true;
        }
    }
}
