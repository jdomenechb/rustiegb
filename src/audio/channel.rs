use crate::audio::registers::{AudioRegister, InitialLengthRegister, WriteEffect};
use crate::memory::memory_sector::ReadMemory;
use crate::utils::math::is_bit_set;
use crate::{Byte, Word};

pub enum ChannelEvent {
    None,
    ChannelEnabled(u8),
    ChannelDisabled(u8),
}

pub struct Channel<
    T: AudioRegister,
    U: AudioRegister + InitialLengthRegister,
    V: AudioRegister,
    X: AudioRegister,
    Y: AudioRegister,
> {
    number: u8,

    nrx0: T,
    nrx1: U,
    nrx2: V,
    nrx3: X,
    nrx4: Y,

    length_counter: Byte,
    max_length: Byte,
}

impl<
    T: AudioRegister,
    U: AudioRegister + InitialLengthRegister,
    V: AudioRegister,
    X: AudioRegister,
    Y: AudioRegister,
> Channel<T, U, V, X, Y>
{
    pub fn new(number: u8, nrx0: T, nrx1: U, nrx2: V, nrx3: X, nrx4: Y, max_length: Byte) -> Self {
        Self {
            number,
            nrx0,
            nrx1,
            nrx2,
            nrx3,
            nrx4,
            length_counter: 0,
            max_length,
        }
    }

    pub fn clear(&mut self) {
        self.nrx0.clear();
        self.nrx1.clear();
        self.nrx2.clear();
        self.nrx3.clear();
        self.nrx4.clear();
    }

    pub fn write_byte(&mut self, position: Word, value: Byte) -> ChannelEvent {
        let write_event = match position {
            0 => self.nrx0.write(value),
            1 => self.nrx1.write(value),
            2 => self.nrx2.write(value),
            3 => self.nrx3.write(value),
            4 => self.nrx4.write(value),
            _ => unreachable!("Write address {position:X} not supported for channel"),
        };

        match write_event {
            WriteEffect::None => ChannelEvent::None,
            WriteEffect::Triggered => {
                if self.length_counter == self.max_length {
                    self.length_counter = self.nrx1.get_initial_length();
                }

                ChannelEvent::ChannelEnabled(self.number)
            }
            WriteEffect::DacDisabled => ChannelEvent::ChannelDisabled(self.number),
            WriteEffect::AudioOff => unreachable!("Audio off is not supported for channel"),
        }
    }

    pub fn tick_length(&mut self) -> ChannelEvent {
        if !self.is_length_enabled() {
            return ChannelEvent::None;
        }

        self.length_counter = self.length_counter.wrapping_add(1);

        if self.length_counter == self.max_length {
            return ChannelEvent::ChannelDisabled(self.number);
        }

        ChannelEvent::None
    }

    fn is_length_enabled(&self) -> bool {
        is_bit_set(&self.nrx4.read(), 6)
    }
}

impl<
    T: AudioRegister,
    U: AudioRegister + InitialLengthRegister,
    V: AudioRegister,
    X: AudioRegister,
    Y: AudioRegister,
> ReadMemory for Channel<T, U, V, X, Y>
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
