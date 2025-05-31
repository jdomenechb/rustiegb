use crate::Byte;

#[derive(Default, Clone, Debug)]
pub struct LY {
    pub value: Byte,
}

impl LY {
    pub(crate) fn increment(&mut self) {
        self.value += 1
    }

    pub(crate) fn reset(&mut self) {
        self.value = 0
    }

    pub fn has_reached_end_of_screen(&self) -> bool {
        self.value >= 143
    }

    pub fn has_reached_end_of_vblank(&self) -> bool {
        self.value > 153
    }
}
