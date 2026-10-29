use crate::{Byte, Word};

#[derive(Default)]
#[readonly::make]
pub struct Div {
    value: Word,
}

impl Div {
    pub fn single_step(&mut self) {
        self.value = self.value.wrapping_add(1);
    }

    fn reset_value(&mut self) {
        self.value = 0x00;
    }

    pub fn write(&mut self) {
        self.reset_value()
    }

    pub fn read(&self) -> Byte {
        (self.value >> 8) as Byte
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(div: &mut Div, cycles: u8) {
        for _ in 0..cycles {
            div.single_step();
        }
    }

    #[test]
    fn it_does_not_increase_in_its_maximum_value() {
        let mut div = Div::default();
        step(&mut div, 0xFF);

        assert_eq!(div.read(), 0);
    }
    #[test]
    fn it_increases_with_maximum_value_plus_1() {
        let mut div = Div::default();
        step(&mut div, 0xFF);
        step(&mut div, 0x01);

        assert_eq!(div.read(), 1);
    }

    #[test]
    fn it_keeps_increasing() {
        let mut div = Div::default();
        step(&mut div, 0xFF);
        step(&mut div, 0xFF);
        step(&mut div, 0xFF);

        assert_eq!(div.read(), 2);
    }

    #[test]
    fn it_resets_value() {
        let mut div = Div::default();
        step(&mut div, 0xFF);
        step(&mut div, 0x01);
        div.reset_value();

        assert_eq!(div.read(), 0);
    }

    #[test]
    fn it_resets_value_and_internal_cycles() {
        let mut div = Div::default();
        step(&mut div, 0xFF);
        step(&mut div, 0xFF);
        div.reset_value();
        step(&mut div, 0xFF);

        assert_eq!(div.read(), 0);

        step(&mut div, 0x01);

        assert_eq!(div.read(), 1);
    }
}
