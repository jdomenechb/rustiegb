mod channel_stoppable;
mod control_updatable;
mod envelope_updatable;
mod frequency_updatable;
mod length_updatable;

pub use self::channel_stoppable::ChannelStopabble;
pub use self::control_updatable::ControlRegisterUpdatable;
pub use self::control_updatable::ControlUpdatable;
pub use self::envelope_updatable::EnvelopeRegisterUpdatable;
pub use self::envelope_updatable::EnvelopeUpdatable;
pub use self::frequency_updatable::FrequencyRegisterUpdatable;
pub use self::frequency_updatable::FrequencyUpdatable;
pub use self::length_updatable::LengthRegisterUpdatable;
pub use self::length_updatable::LengthUpdatable;
