use crate::Byte;
use crate::audio::registers::{AudioRegister, WriteEffect};

/// Sweep
/// ```
/// 7 - RW - Clock swift
/// 6 - RW - Clock swift
/// 5 - RW - Clock swift
/// 4 - RW - Clock swift
/// 3 - RW - LFSR width
/// 2 - RW - Clock divider
/// 1 - RW - Clock divider
/// 0 - RW - Clock divider
/// ```
#[derive(Default)]
pub struct NR43 {
    value: Byte,
}

impl NR43 {
    pub fn is_short(&self) -> bool {
        self.value & 0b1000 != 0
    }

    pub fn frequency(&self) -> f32 {
        let shift = self.value >> 4;

        if shift >= 14 {
            return 0.0;
        }

        let divider = match self.value & 0b111 {
            0 => 0.5,
            divider => divider as f32,
        };

        262144.0 / (divider * (1u32 << shift) as f32)
    }
}

impl AudioRegister for NR43 {
    const READ_MASK: Byte = 0;
    const WRITE_MASK: Byte = 0;

    fn set_value(&mut self, value: Byte) -> WriteEffect {
        self.value = value;

        WriteEffect::None
    }

    fn value(&self) -> Byte {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_read_and_write_the_value() {
        let mut fixture = NR43::default();
        fixture.write(0xFF);
        assert_eq!(fixture.read(), 0xFF);

        fixture.write(0x00);
        assert_eq!(fixture.read(), 0x00);
    }

    #[test]
    fn it_calculates_the_frequency() {
        let mut fixture = NR43::default();

        for (value, expected) in [
            (0x00, 524288.0),
            (0x01, 262144.0),
            (0x13, 43690.668),
            (0xD7, 4.5714),
            (0xE0, 0.0),
            (0xF7, 0.0),
        ] {
            fixture.write(value);
            assert!((fixture.frequency() - expected).abs() < 0.01, "{value:X}");
        }
    }

    #[test]
    fn it_reports_the_lfsr_width() {
        let mut fixture = NR43::default();

        fixture.write(0b0000_1000);
        assert!(fixture.is_short());

        fixture.write(0b1111_0111);
        assert!(!fixture.is_short());
    }
}
