use crate::audio::registers::WriteEffect;
use crate::memory::memory_sector::ReadMemory;
use crate::{Byte, Word};
use thiserror::Error;

pub enum ChannelEvent {
    None(Option<WriteEffect>),
    ChannelEnabled(u8, Option<WriteEffect>),
    ChannelDisabled(u8, Option<WriteEffect>),
}

#[derive(Error, Debug)]
pub enum ChannelEventTryFormError {
    #[error("ChannelEvent is not a WriteEffect")]
    NoWriteEffect,
}

impl TryFrom<&ChannelEvent> for WriteEffect {
    type Error = ChannelEventTryFormError;

    fn try_from(value: &ChannelEvent) -> Result<Self, Self::Error> {
        match value {
            ChannelEvent::None(Some(effect)) => Ok(*effect),
            ChannelEvent::ChannelEnabled(_, Some(effect)) => Ok(*effect),
            ChannelEvent::ChannelDisabled(_, Some(effect)) => Ok(*effect),
            ChannelEvent::None(None) => Err(ChannelEventTryFormError::NoWriteEffect),
            ChannelEvent::ChannelEnabled(_, None) => Err(ChannelEventTryFormError::NoWriteEffect),
            ChannelEvent::ChannelDisabled(_, None) => Err(ChannelEventTryFormError::NoWriteEffect),
        }
    }
}

pub trait Channel: ReadMemory {
    fn clear(&mut self);
    fn write_byte(&mut self, position: Word, value: Byte, div_apu: &Byte) -> ChannelEvent;
    fn tick_length(&mut self) -> ChannelEvent;
}
