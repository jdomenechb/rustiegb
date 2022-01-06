#[derive(Eq, PartialEq, Copy, Clone)]
pub enum VolumeEnvelopeDirection {
    Up,
    Down,
}

impl Default for VolumeEnvelopeDirection {
    fn default() -> Self {
        VolumeEnvelopeDirection::Up
    }
}

impl From<bool> for VolumeEnvelopeDirection {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Down,
            true => Self::Up,
        }
    }
}
