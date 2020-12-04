use crate::Byte;

#[derive(Default, Clone)]
pub struct LY {
    ly: Byte,
}

impl LY {
    pub(in crate::memory) fn increment(&mut self) {
        self.ly += 1
    }

    pub(in crate::memory) fn reset(&mut self) {
        self.ly = 0
    }

    pub fn has_reached_end_of_screen(&self) -> bool {
        self.ly == 143
    }

    pub fn has_reached_end_of_vblank(&self) -> bool {
        self.ly > 153
    }
}

impl From<Byte> for LY {
    fn from(value: Byte) -> Self {
        Self { ly: value }
    }
}

impl From<LY> for Byte {
    fn from(original: LY) -> Self {
        original.ly
    }
}
