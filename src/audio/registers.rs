use crate::Byte;

pub mod nr10;

pub mod nr30;
pub mod nr31;
pub mod nr32;

pub mod no_register;
pub mod nr41;
pub mod nr43;
pub mod nr44;
pub mod nr52;
pub mod nrx1;
pub mod nrx2;
pub mod nrx3;
pub mod nrx4;

pub enum WriteEffect {
    None,
    Triggered,
    DacDisabled,
    AudioOff,
    NRX1Updated,
}

pub trait AudioRegister {
    const READ_MASK: Byte;
    const WRITE_MASK: Byte;

    fn set_value(&mut self, value: Byte) -> WriteEffect;
    fn value(&self) -> Byte;

    fn read(&self) -> Byte {
        self.value() | Self::READ_MASK
    }

    fn write(&mut self, value: Byte) -> WriteEffect {
        self.set_value(value | Self::WRITE_MASK)
    }

    fn clear(&mut self) {
        self.set_value(0x0);
    }
}

pub trait InitialLengthRegister {
    fn get_initial_length(&self) -> Byte;
}

pub trait DacRegister {
    fn is_dac_enabled(&self) -> bool;
}
