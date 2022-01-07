use crate::audio::volume_envelope::VolumeEnvelopeDirection;
use crate::Byte;

#[derive(Default)]
pub struct VolumeEnvelopeDescription {
    pub initial_volume: Byte,
    pub current_volume: Byte,
    pub direction: VolumeEnvelopeDirection,
    pub period: u8,
    pub period_timer: u8,
}

impl VolumeEnvelopeDescription {
    pub fn new(initial_volume: Byte, direction: VolumeEnvelopeDirection, period: u8) -> Self {
        Self {
            initial_volume,
            current_volume: initial_volume,
            direction,
            period,
            period_timer: period,
        }
    }

    pub fn step_64(&mut self) {
        if self.period == 0 {
            return;
        }

        if self.period_timer != 0 {
            self.period_timer -= 1;

            if self.period_timer == 0 {
                self.period_timer = self.period;

                match self.direction {
                    VolumeEnvelopeDirection::Up => {
                        if self.current_volume < 0xF {
                            self.current_volume += 1;
                        }
                    }
                    VolumeEnvelopeDirection::Down => {
                        if self.current_volume > 0 {
                            self.current_volume -= 1;
                        }
                    }
                }
            }
        }
    }
}

impl From<Byte> for VolumeEnvelopeDescription {
    fn from(register: Byte) -> Self {
        let initial_volume = register >> 4;
        let period = register & 0b111;

        VolumeEnvelopeDescription {
            initial_volume,
            current_volume: initial_volume,
            direction: VolumeEnvelopeDirection::from(register & 0b1000 == 0b1000),
            period,
            period_timer: period,
        }
    }
}
