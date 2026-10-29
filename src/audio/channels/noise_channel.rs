use crate::audio::channels::channel::{Channel, ChannelEvent, ChannelSignal};
use crate::audio::channels::default_channel::DefaultChannel;
use crate::audio::channels::envelope::Envelope;
use crate::audio::registers::WriteEffect;
use crate::audio::registers::no_register::NoRegister;
use crate::audio::registers::nr41::NR41;
use crate::audio::registers::nr43::NR43;
use crate::audio::registers::nr44::NR44;
use crate::audio::registers::nrx2::NRX2;
use crate::memory::memory_sector::ReadMemory;
use crate::{Byte, Word};

pub struct NoiseChannel {
    channel: DefaultChannel<NoRegister, NR41, NRX2, NR43, NR44>,
    envelope: Envelope,
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            channel: DefaultChannel::new(
                4,
                NoRegister::default(),
                NR41::default(),
                NRX2::new_nr42(),
                NR43::default(),
                NR44::default(),
                64,
                false,
            ),
            envelope: Envelope::new(0),
        }
    }
}

impl ReadMemory for NoiseChannel {
    fn read_byte(&self, position: Word) -> Byte {
        self.channel.read_byte(position)
    }
}

impl Channel for NoiseChannel {
    fn clear(&mut self) -> ChannelEvent {
        self.envelope.clear();
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
            self.envelope.trigger(self.channel.get_nrx2());
        }

        (channel_event, write_effect)
    }

    fn tick_length(&mut self) -> ChannelEvent {
        self.channel.tick_length()
    }

    fn single_step(&mut self) {
        self.channel.single_step()
    }

    fn tick_envelope(&mut self) {
        self.envelope.tick(self.channel.get_nrx2());
    }

    fn is_dac_on(&self) -> bool {
        self.channel.is_dac_on()
    }

    fn output_signal(&self) -> ChannelSignal {
        let nr43 = self.channel.get_nrx3();

        ChannelSignal::Noise {
            frequency: nr43.frequency(),
            is_short: nr43.is_short(),
            volume: self.envelope.output_volume(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_outputs_the_frequency_width_and_volume() {
        let mut fixture = NoiseChannel::new();
        fixture.write_byte(2, 0xA0, &0, false);
        fixture.write_byte(3, 0b0000_1001, &0, false);
        fixture.write_byte(4, 0x80, &0, false);

        let ChannelSignal::Noise {
            frequency,
            is_short,
            volume,
        } = fixture.output_signal()
        else {
            panic!("Expected a noise signal");
        };

        assert_eq!(frequency, 262144.0 / 1.0);
        assert!(is_short);
        assert!((volume - 10.0 / 15.0).abs() < 0.001);
    }
}
