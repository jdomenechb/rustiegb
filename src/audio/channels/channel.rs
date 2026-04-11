use crate::memory::memory_sector::ReadMemory;
use crate::{Byte, Word};

pub enum ChannelEvent {
    None,
    ChannelEnabled(u8),
    ChannelDisabled(u8),
}

pub trait Channel: ReadMemory {
    fn clear(&mut self);
    fn write_byte(&mut self, position: Word, value: Byte, div_apu: &Byte) -> ChannelEvent;
    fn tick_length(&mut self) -> ChannelEvent;

    fn process_triggered_write_effect(&mut self, makes_length_tick: bool) -> ChannelEvent;
}
