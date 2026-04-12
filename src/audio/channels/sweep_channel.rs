use crate::audio::channels::channel::{Channel, ChannelEvent};
use crate::audio::channels::default_channel::DefaultChannel;
use crate::audio::registers::WriteEffect;
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
    sweep_ticks_left: Byte,
    sweep_frequency_shadow_register: u32,
    at_least_one_sweep_negate_has_been_calculated_since_last_trigger: bool,
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
            sweep_ticks_left: 0,
            sweep_frequency_shadow_register: 0,
            at_least_one_sweep_negate_has_been_calculated_since_last_trigger: false,
        }
    }
}

impl SweepChannel {
    pub fn tick_sweep(&mut self) -> ChannelEvent {
        // Prevents overflows
        if self.sweep_ticks_left == 0 {
            return ChannelEvent::None;
        };

        let ticks_exhausted = self.decrease_sweep_ticks_left();

        if !self.sweep_enabled || self.channel.get_nrx0().read_pace() == 0 {
            return ChannelEvent::None;
        }

        let (new_frequency, frequency_will_overflow) = self.calculate_new_frequency();

        if frequency_will_overflow {
            return ChannelEvent::ChannelDisabled(self.channel.get_number());
        }

        if !ticks_exhausted {
            return ChannelEvent::None;
        }

        if self.channel.get_nrx0().read_step() != 0 {
            self.sweep_frequency_shadow_register = new_frequency;
            self.channel.write_frequency(new_frequency);

            let (_, frequency_will_overflow) = self.calculate_new_frequency();

            if frequency_will_overflow {
                return ChannelEvent::ChannelDisabled(self.channel.get_number());
            }
        }

        ChannelEvent::None
    }

    // Returns true if sweep is reset
    fn decrease_sweep_ticks_left(&mut self) -> bool {
        self.sweep_ticks_left = self.sweep_ticks_left - 1;

        if self.sweep_ticks_left == 0 {
            self.reset_sweep_ticks_left();
            return true;
        }

        false
    }

    fn refresh_sweep_enabled(&mut self) {
        let nr10 = self.channel.get_nrx0();
        self.sweep_enabled = nr10.read_pace() != 0 || nr10.read_step() != 0;
    }

    fn reset_sweep_ticks_left(&mut self) {
        self.sweep_ticks_left = self.channel.get_nrx0().read_pace();

        if self.sweep_ticks_left == 0 {
            self.sweep_ticks_left = 8;
        }
    }

    fn calculate_new_frequency(&mut self) -> (u32, bool) {
        let nr10 = self.channel.get_nrx0();

        let frequency = self.sweep_frequency_shadow_register;
        let direction = nr10.read_direction();
        let step = nr10.read_step() as u32;

        let to_add_or_sub = frequency / 2_u32.pow(step);

        let new_frequency = match direction {
            SweepDirection::Add => frequency.wrapping_add(to_add_or_sub),
            SweepDirection::Sub => {
                self.at_least_one_sweep_negate_has_been_calculated_since_last_trigger = true;
                frequency.wrapping_sub(to_add_or_sub)
            }
        };

        (new_frequency, Self::frequency_will_overflow(new_frequency))
    }

    fn frequency_will_overflow(new_frequency: u32) -> bool {
        new_frequency > 0x7FF
    }

    fn process_triggered_write_effect(&mut self, channel_event: ChannelEvent) -> ChannelEvent {
        self.sweep_frequency_shadow_register = self.channel.read_frequency();

        self.reset_sweep_ticks_left();
        self.refresh_sweep_enabled();

        if self.channel.get_nrx0().read_step() != 0 {
            let (_, frequency_will_overflow) = self.calculate_new_frequency();

            if frequency_will_overflow {
                return ChannelEvent::ChannelDisabled(self.channel.get_number());
            }
        }

        channel_event
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

    fn write_byte(
        &mut self,
        position: u16,
        value: u8,
        div_apu: &u8,
    ) -> (ChannelEvent, WriteEffect) {
        let (channel_event, write_effect) = self.channel.write_byte(position, value, div_apu);

        let new_channel_event = match write_effect {
            WriteEffect::Triggered => self.process_triggered_write_effect(channel_event),
            WriteEffect::SweepDirectionFromSubToAdd => {
                if self.at_least_one_sweep_negate_has_been_calculated_since_last_trigger {
                    ChannelEvent::ChannelDisabled(self.channel.get_number())
                } else {
                    channel_event
                }
            }
            _ => channel_event,
        };

        if position == 0 {
            self.at_least_one_sweep_negate_has_been_calculated_since_last_trigger = false;
        }

        (new_channel_event, write_effect)
    }

    fn tick_length(&mut self) -> ChannelEvent {
        self.channel.tick_length()
    }
}
