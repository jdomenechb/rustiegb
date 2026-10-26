use crate::audio::channels::channel::{Channel, ChannelEvent};
use crate::audio::registers::{
    AudioRegister, InitialLengthRegister, TriggerableRegister, WriteEffect,
};
use crate::memory::memory_sector::ReadMemory;
use crate::utils::math::is_bit_set;
use crate::{Byte, Word};

pub struct DefaultChannel<
    T: AudioRegister,
    U: AudioRegister + InitialLengthRegister,
    V: AudioRegister,
    X: AudioRegister,
    Y: AudioRegister + TriggerableRegister,
> {
    number: u8,

    nrx0: T,
    nrx1: U,
    nrx2: V,
    nrx3: X,
    nrx4: Y,

    length_counter: Word,
    max_length: Word,

    dac: bool,
}

impl<
    T: AudioRegister,
    U: AudioRegister + InitialLengthRegister,
    V: AudioRegister,
    X: AudioRegister,
    Y: AudioRegister + TriggerableRegister,
> DefaultChannel<T, U, V, X, Y>
{
    pub fn new(number: u8, nrx0: T, nrx1: U, nrx2: V, nrx3: X, nrx4: Y, max_length: Word) -> Self {
        Self {
            number,
            nrx0,
            nrx1,
            nrx2,
            nrx3,
            nrx4,
            length_counter: 0,
            max_length,
            dac: true,
        }
    }

    pub fn get_number(&self) -> u8 {
        self.number
    }

    pub fn get_nrx0(&self) -> &T {
        &self.nrx0
    }

    fn is_length_enabled(&self) -> bool {
        is_bit_set(&self.nrx4.read(), 6)
    }

    pub fn read_frequency(&self) -> u32 {
        (((self.nrx4.value() as u32) & 0b111) << 8) | self.nrx3.value() as u32
    }

    pub fn write_frequency(&mut self, new_value: u32) {
        self.nrx3.write(new_value as Byte);

        let new_value = (self.nrx4.value() & 0b1111_1000) | ((new_value >> 8) as Byte);
        self.nrx4.write(new_value);
    }
}

impl<
    T: AudioRegister,
    U: AudioRegister + InitialLengthRegister,
    V: AudioRegister,
    X: AudioRegister,
    Y: AudioRegister + TriggerableRegister,
> Channel for DefaultChannel<T, U, V, X, Y>
{
    fn clear(&mut self) -> ChannelEvent {
        let write_effects = [
            self.nrx0.clear(),
            self.nrx1.clear(),
            self.nrx2.clear(),
            self.nrx3.clear(),
            self.nrx4.clear(),
        ];

        if write_effects.contains(&WriteEffect::DacDisabled) {
            self.dac = false;
            return ChannelEvent::ChannelDisabled(self.number);
        }

        ChannelEvent::None
    }

    fn write_byte(
        &mut self,
        position: Word,
        value: Byte,
        div_apu: &Byte,
        _is_channel_on: bool,
    ) -> (ChannelEvent, WriteEffect) {
        let makes_length_tick = div_apu.is_multiple_of(2);

        let write_effect = match position {
            0 => self.nrx0.write(value),
            1 => self.nrx1.write(value),
            2 => self.nrx2.write(value),
            3 => self.nrx3.write(value),
            4 => {
                let was_length_disabled = !self.is_length_enabled();
                let write_effect = self.nrx4.write(value);
                let is_length_enabled = self.is_length_enabled();

                if !makes_length_tick
                    && is_length_enabled
                    && was_length_disabled
                    && self.length_counter != 0
                {
                    self.length_counter = self.length_counter.wrapping_sub(1);

                    if self.length_counter == 0 && !self.nrx4.is_triggered() {
                        return (ChannelEvent::ChannelDisabled(self.number), write_effect);
                    }
                }

                write_effect
            }
            _ => unreachable!("Write address {position:X} not supported for channel"),
        };

        match write_effect {
            WriteEffect::None => (ChannelEvent::None, write_effect),
            WriteEffect::Triggered => {
                if self.length_counter == 0 {
                    self.length_counter = self.max_length;

                    if !makes_length_tick && self.is_length_enabled() {
                        self.length_counter = self.length_counter.wrapping_sub(1);
                    }
                }

                if !self.dac {
                    return (ChannelEvent::None, write_effect);
                }

                (ChannelEvent::ChannelEnabled(self.number), write_effect)
            }
            WriteEffect::DacDisabled => {
                self.dac = false;
                (ChannelEvent::ChannelDisabled(self.number), write_effect)
            }
            WriteEffect::DacEnabled => {
                self.dac = true;
                (ChannelEvent::None, write_effect)
            }
            WriteEffect::NRX1Updated => {
                self.length_counter = self.max_length - self.nrx1.get_initial_length() as Word;
                (ChannelEvent::None, write_effect)
            }
            WriteEffect::AudioOff | WriteEffect::AudioOn => {
                unreachable!("Audio power effects not supported for channel")
            }
            WriteEffect::SweepDirectionFromSubToAdd => (ChannelEvent::None, write_effect),
        }
    }

    fn tick_length(&mut self) -> ChannelEvent {
        if !self.is_length_enabled() {
            return ChannelEvent::None;
        }

        self.length_counter = self.length_counter.wrapping_sub(1);

        if self.length_counter == 0 {
            return ChannelEvent::ChannelDisabled(self.number);
        }

        ChannelEvent::None
    }

    fn single_step(&mut self) {}
}

impl<
    T: AudioRegister,
    U: AudioRegister + InitialLengthRegister,
    V: AudioRegister,
    X: AudioRegister,
    Y: AudioRegister + TriggerableRegister,
> ReadMemory for DefaultChannel<T, U, V, X, Y>
{
    fn read_byte(&self, position: Word) -> Byte {
        match position {
            0 => self.nrx0.read(),
            1 => self.nrx1.read(),
            2 => self.nrx2.read(),
            3 => self.nrx3.read(),
            4 => self.nrx4.read(),
            _ => unreachable!("Read address {position:X} not supported for channel"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::registers::no_register::NoRegister;
    use crate::audio::registers::nr30::NR30;
    use crate::audio::registers::nr31::NR31;
    use crate::audio::registers::nr32::NR32;
    use crate::audio::registers::nrx1::NRX1;
    use crate::audio::registers::nrx2::NRX2;
    use crate::audio::registers::nrx3::NRX3;
    use crate::audio::registers::nrx4::NRX4;

    #[test]
    fn it_keeps_length_and_clears_the_rest_when_audio_is_off() {
        let mut fixture = DefaultChannel::new(
            2,
            NoRegister::default(),
            NRX1::new_nr21(),
            NRX2::new_nr22(),
            NRX3::default(),
            NRX4::default(),
            64,
        );

        fixture.write_byte(1, 0xFF, &0, false);
        fixture.write_byte(2, 0xFF, &0, false);
        fixture.write_byte(3, 0xFF, &0, false);
        fixture.write_byte(4, 0x07, &0, false);

        fixture.clear();

        assert_eq!(fixture.length_counter, 1);
        assert_eq!(fixture.nrx1.get_initial_length(), 0b11_1111);
        assert_eq!(fixture.read_byte(1), 0x3F);
        assert_eq!(fixture.read_byte(2), 0x00);
        assert_eq!(fixture.read_byte(3), 0xFF);
        assert_eq!(fixture.read_byte(4), 0xBF);
    }

    #[test]
    fn it_keeps_length_of_wave_channel_when_audio_is_off() {
        let mut fixture = DefaultChannel::new(
            3,
            NoRegister::default(),
            NR31::default(),
            NoRegister::default(),
            NRX3::default(),
            NRX4::default(),
            256,
        );

        fixture.write_byte(1, 0x10, &0, false);

        fixture.clear();

        assert_eq!(fixture.length_counter, 256 - 0x10);
        assert_eq!(fixture.nrx1.get_initial_length(), 0x10);
    }

    fn new_square_channel() -> DefaultChannel<NoRegister, NRX1, NRX2, NRX3, NRX4> {
        DefaultChannel::new(
            2,
            NoRegister::default(),
            NRX1::new_nr21(),
            NRX2::new_nr22(),
            NRX3::default(),
            NRX4::default(),
            64,
        )
    }

    #[test]
    fn it_disables_the_channel_when_cleared_with_the_dac_on() {
        let mut fixture = new_square_channel();
        fixture.write_byte(2, 0xFF, &0, false);

        let event = fixture.clear();

        assert!(matches!(event, ChannelEvent::ChannelDisabled(2)));
        assert_eq!(fixture.read_byte(2), 0x00);
    }

    #[test]
    fn it_does_not_enable_the_channel_on_trigger_after_being_cleared() {
        let mut fixture = new_square_channel();
        fixture.write_byte(2, 0xFF, &0, false);
        fixture.clear();

        let (event, write_effect) = fixture.write_byte(4, 0x80, &0, false);

        assert!(matches!(event, ChannelEvent::None));
        assert!(write_effect == WriteEffect::Triggered);
    }

    #[test]
    fn it_enables_the_channel_on_trigger_when_the_dac_is_turned_on_after_clear() {
        let mut fixture = new_square_channel();
        fixture.write_byte(2, 0xFF, &0, false);
        fixture.clear();
        fixture.write_byte(2, 0x08, &0, false);

        let (event, _) = fixture.write_byte(4, 0x80, &0, false);

        assert!(matches!(event, ChannelEvent::ChannelEnabled(2)));
    }

    #[test]
    fn it_disables_the_wave_channel_when_cleared_with_the_dac_on() {
        let mut fixture = DefaultChannel::new(
            3,
            NR30::default(),
            NR31::default(),
            NR32::default(),
            NRX3::default(),
            NRX4::default(),
            256,
        );
        fixture.write_byte(0, 0x80, &0, false);

        let event = fixture.clear();

        assert!(matches!(event, ChannelEvent::ChannelDisabled(3)));

        let (event, _) = fixture.write_byte(4, 0x80, &0, false);

        assert!(matches!(event, ChannelEvent::None));
    }
}
