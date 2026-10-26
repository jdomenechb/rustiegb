use crate::audio::registers::WriteEffect;
use crate::memory::memory_sector::ReadMemory;
use crate::{Byte, Word};

pub enum ChannelEvent {
    None,
    ChannelEnabled(u8),
    ChannelDisabled(u8),
}

pub trait Channel: ReadMemory {
    fn clear(&mut self) -> ChannelEvent;

    fn write_byte(
        &mut self,
        position: Word,
        value: Byte,
        div_apu: &Byte,
        is_channel_on: bool,
    ) -> (ChannelEvent, WriteEffect);

    fn tick_length(&mut self) -> ChannelEvent;

    fn single_step(&mut self);
}
