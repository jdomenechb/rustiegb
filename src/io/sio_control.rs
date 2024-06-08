use crate::Byte;

#[derive(Clone)]
pub struct SioControl {
    value: Byte,
}

impl From<Byte> for SioControl {
    fn from(value: Byte) -> Self {
        Self {
            value: value | 0b1111110,
        }
    }
}

impl From<&SioControl> for Byte {
    fn from(original: &SioControl) -> Self {
        original.value
    }
}

impl Default for SioControl {
    fn default() -> Self {
        Self::from(0)
    }
}

#[cfg(test)]
mod tests {
    use crate::Byte;
    use crate::io::sio_control::SioControl;

    #[test]
    fn test_ok() {
        assert_eq!(Byte::from(&SioControl::from(0x00)), 0b1111110);
    }
}
