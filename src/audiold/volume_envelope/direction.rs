#[derive(Eq, PartialEq, Copy, Clone, Default)]
pub enum VolumeEnvelopeDirection {
    #[default]
    Up,
    Down,
}

impl From<bool> for VolumeEnvelopeDirection {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Down,
            true => Self::Up,
        }
    }
}
