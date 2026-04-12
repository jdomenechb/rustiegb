use crate::Byte;
use crate::audio::registers::{AudioRegister, WriteEffect};

#[derive(PartialEq)]
pub enum SweepDirection {
    Add,
    Sub,
}

/// Sweep
/// ```
/// 7
/// 6 - RW - Pace
/// 5 - RW - Pace
/// 4 - RW - Pace
/// 3 - RW - Direction
/// 2 - RW - Individual step
/// 1 - RW - Individual step
/// 0 - RW - Individual step
/// ```
pub struct NR10 {
    value: Byte,
}

impl NR10 {
    pub fn read_pace(&self) -> Byte {
        (self.value >> 4) & 0b111
    }

    pub fn read_direction(&self) -> SweepDirection {
        let value = (self.value & 0b0000_1000) >> 3;

        if value == 0 {
            SweepDirection::Add
        } else {
            SweepDirection::Sub
        }
    }

    pub fn read_step(&self) -> Byte {
        self.value & 0b0000_0111
    }
}

impl AudioRegister for NR10 {
    const READ_MASK: Byte = 0b1000_0000;
    const WRITE_MASK: Byte = 0b1000_0000;

    fn set_value(&mut self, value: Byte) -> WriteEffect {
        let old_direction = self.read_direction();

        self.value = value;
        let new_direction = self.read_direction();

        if old_direction == SweepDirection::Sub && new_direction == SweepDirection::Add {
            return WriteEffect::SweepDirectionFromSubToAdd;
        }

        WriteEffect::None
    }

    fn value(&self) -> Byte {
        self.value
    }
}

impl Default for NR10 {
    fn default() -> Self {
        Self { value: 0x80 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_read_and_write_the_value() {
        let mut fixture = NR10::default();
        fixture.write(0xFF);
        assert_eq!(fixture.read(), 0xFF);

        fixture.write(0x00);
        assert_eq!(fixture.read(), 0x80);
    }
}
