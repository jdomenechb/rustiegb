use crate::audio::channels::channel::{Channel, ChannelEvent, ChannelSignal};
use crate::audio::channels::default_channel::DefaultChannel;
use crate::audio::registers::WriteEffect;
use crate::audio::registers::no_register::NoRegister;
use crate::audio::registers::nrx1::NRX1;
use crate::audio::registers::nrx2::NRX2;
use crate::audio::registers::nrx3::NRX3;
use crate::audio::registers::nrx4::NRX4;
use crate::memory::memory_sector::ReadMemory;
use crate::{Byte, Word};

pub struct PulseChannel {
    channel: DefaultChannel<NoRegister, NRX1, NRX2, NRX3, NRX4>,
}

impl PulseChannel {
    pub fn new() -> Self {
        Self {
            channel: DefaultChannel::new(
                2,
                NoRegister::default(),
                NRX1::new_nr21(),
                NRX2::new_nr22(),
                NRX3::default(),
                NRX4::default(),
                64,
            ),
        }
    }
}

impl ReadMemory for PulseChannel {
    fn read_byte(&self, position: Word) -> Byte {
        self.channel.read_byte(position)
    }
}

impl Channel for PulseChannel {
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
        self.channel
            .write_byte(position, value, div_apu, is_channel_on)
    }

    fn tick_length(&mut self) -> ChannelEvent {
        self.channel.tick_length()
    }

    fn single_step(&mut self) {
        self.channel.single_step()
    }

    fn is_dac_on(&self) -> bool {
        self.channel.is_dac_on()
    }

    fn output_signal(&self) -> ChannelSignal {
        let frequency = 131072.0 / (2048.0 - self.channel.read_frequency() as f32);

        ChannelSignal::Pulse {
            frequency,
            wave_duty: self.channel.get_nrx1().output_wave_duty(),
        }
    }
}
