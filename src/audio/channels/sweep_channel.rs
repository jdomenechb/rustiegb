use crate::audio::channels::channel::{Channel, ChannelEvent};
use crate::audio::channels::default_channel::DefaultChannel;
use crate::audio::registers::nr10::{NR10, SweepDirection};
use crate::audio::registers::nrx1::NRX1;
use crate::audio::registers::nrx2::NRX2;
use crate::audio::registers::nrx3::NRX3;
use crate::audio::registers::nrx4::NRX4;
use crate::memory::memory_sector::ReadMemory;
use crate::{Byte, Word};

pub struct SweepChannel {
    channel: DefaultChannel<NR10, NRX1, NRX2, NRX3, NRX4>,

    sweep_enabled: bool,
    sweep_pace: Byte,
    sweep_ticks_accumulated: Byte,
    sweep_frequency_shadow_register: u32,
}

impl SweepChannel {
    pub fn new() -> Self {
        Self {
            channel: DefaultChannel::new(
                1,
                NR10::default(),
                NRX1::new_nr11(),
                NRX2::new_nr12(),
                NRX3::default(),
                NRX4::default(),
                64,
            ),
            sweep_enabled: false,
            sweep_pace: 0,
            sweep_ticks_accumulated: 0,
            sweep_frequency_shadow_register: 0,
        }
    }
}

impl SweepChannel {
    pub fn tick_sweep(&mut self) -> ChannelEvent {
        if !self.sweep_enabled || self.sweep_pace == 0 {
            return ChannelEvent::None;
        }

        let new_frequency = self.calculate_new_frequency();

        if Self::frequency_will_overflow(new_frequency) {
            return ChannelEvent::ChannelDisabled(self.channel.get_number());
        }

        self.sweep_ticks_accumulated = (self.sweep_ticks_accumulated + 1) % self.sweep_pace;

        if self.sweep_ticks_accumulated == 0 {
            self.sweep_frequency_shadow_register = new_frequency;
            self.channel.write_frequency(new_frequency);

            if Self::frequency_will_overflow(self.calculate_new_frequency()) {
                return ChannelEvent::ChannelDisabled(self.channel.get_number());
            }
        }

        self.refresh_sweep_pace();

        ChannelEvent::None
    }

    fn refresh_sweep_pace(&mut self) {
        self.sweep_pace = (self.read_byte(1) & 0b0111_0000) >> 4;
        self.sweep_ticks_accumulated = 0;
    }

    fn calculate_new_frequency(&self) -> u32 {
        let nr10 = self.channel.get_nrx0();

        let frequency = self.sweep_frequency_shadow_register;
        let direction = nr10.read_direction();
        let step = nr10.read_step() as u32;

        let to_add_or_sub = frequency / 2 ^ step;

        match direction {
            SweepDirection::Add => frequency.wrapping_add(to_add_or_sub),
            SweepDirection::Sub => frequency.wrapping_sub(to_add_or_sub),
        }
    }

    fn frequency_will_overflow(new_frequency: u32) -> bool {
        new_frequency > 0x7FF
    }
}

impl ReadMemory for SweepChannel {
    fn read_byte(&self, position: Word) -> Byte {
        self.channel.read_byte(position)
    }
}

impl Channel for SweepChannel {
    fn clear(&mut self) {
        self.channel.clear();
    }

    fn write_byte(&mut self, position: u16, value: u8, div_apu: &u8) -> ChannelEvent {
        self.channel.write_byte(position, value, div_apu)
    }

    fn tick_length(&mut self) -> ChannelEvent {
        self.channel.tick_length()
    }

    fn process_triggered_write_effect(&mut self, makes_length_tick: bool) -> ChannelEvent {
        let channel_event = self
            .channel
            .process_triggered_write_effect(makes_length_tick);
        let nr10 = self.channel.get_nrx0();
        let step_is_non_zero = nr10.read_step() != 0;

        self.sweep_frequency_shadow_register = self.channel.read_frequency();
        self.sweep_enabled = nr10.read_pace() != 0 || step_is_non_zero;

        self.refresh_sweep_pace();

        if step_is_non_zero && Self::frequency_will_overflow(self.calculate_new_frequency()) {
            return ChannelEvent::ChannelDisabled(self.channel.get_number());
        }

        channel_event
    }
}
