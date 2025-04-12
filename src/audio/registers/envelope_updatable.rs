use crate::Byte;
use crate::audio::registers::channel_stoppable::ChannelStopabble;
use crate::audio::volume_envelope::VolumeEnvelopeDescription;

pub trait EnvelopeUpdatable: ChannelStopabble {
    fn set_envelope(&mut self, envelope: VolumeEnvelopeDescription);
}

pub trait EnvelopeRegisterUpdatable: EnvelopeUpdatable {
    fn trigger_envelope_register_update(&mut self, register: Byte) {
        let envelope_description = VolumeEnvelopeDescription::from(register);

        if envelope_description.is_disabled() {
            self.stop_channel();
        }

        self.set_envelope(envelope_description);
    }
}
