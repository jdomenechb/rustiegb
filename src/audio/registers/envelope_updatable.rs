use crate::audio::volume_envelope::VolumeEnvelopeDescription;
use crate::Byte;

pub trait EnvelopeUpdatable {
    fn set_envelope(&mut self, envelope: VolumeEnvelopeDescription);
}

pub trait EnvelopeRegisterUpdatable: EnvelopeUpdatable {
    fn trigger_envelope_register_update(&mut self, register: Byte) {
        self.set_envelope(register.into());
    }
}
