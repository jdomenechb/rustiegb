#[derive(Eq, PartialEq, Clone, Copy)]
pub enum WaveOutputLevel {
    Mute,
    Vol100Percent,
    Vol50Percent,
    Vol25Percent,
}

impl From<WaveOutputLevel> for f32 {
    fn from(wol: WaveOutputLevel) -> Self {
        match wol {
            WaveOutputLevel::Mute => 0.0,
            WaveOutputLevel::Vol25Percent => 0.25,
            WaveOutputLevel::Vol50Percent => 0.5,
            WaveOutputLevel::Vol100Percent => 1.0,
        }
    }
}
