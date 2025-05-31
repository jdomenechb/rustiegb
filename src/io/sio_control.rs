use crate::Byte;

#[derive(Clone, Debug)]
#[readonly::make]
pub struct SioControl {
    pub value: Byte,
}

const MASK: u8 = 0b01111110;

impl SioControl {
    pub(crate) fn update(&mut self, value: Byte) {
        self.value = value | MASK
    }
}

impl Default for SioControl {
    fn default() -> Self {
        Self { value: MASK }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_initializes_with_the_expected_value() {
        let sio_control = SioControl::default();

        assert_eq!(sio_control.value, MASK);
    }

    #[test]
    fn it_sets_all_to_1() {
        let mut sio_control = SioControl::default();

        sio_control.update(0x1);
        assert_eq!(sio_control.value, 0b01111111);
    }

    #[test]
    fn it_sets_all_to_1_but_the_end() {
        let mut sio_control = SioControl::default();

        sio_control.update(0x0);
        assert_eq!(sio_control.value, 0b01111110);
    }
}
