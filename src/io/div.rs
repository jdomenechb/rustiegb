use crate::{Byte, Word};

#[derive(Default)]
#[readonly::make]
pub struct Div {
    value: Word,
    remaining_div_cycles: u16,
}

impl Div {
    pub fn step(&mut self, last_instruction_cycles: u8) {
        self.value = self.value.wrapping_add(last_instruction_cycles as u16);
    }

    fn reset_value(&mut self) {
        // FIXME: Possibly it needs to be reset to 0
        self.value &= 0x00FF;
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

    #[test]
    fn it_does_not_increase_in_its_maximum_value() {
        let mut div = Div::default();
        div.step(0xFF);

        assert_eq!(div.read(), 0);
    }
    #[test]
    fn it_increases_with_maximum_value_plus_1() {
        let mut div = Div::default();
        div.step(0xFF);
        div.step(0x01);

        assert_eq!(div.read(), 1);
    }

    #[test]
    fn it_keeps_increasing() {
        let mut div = Div::default();
        div.step(0xFF);
        div.step(0xFF);
        div.step(0xFF);

        assert_eq!(div.read(), 2);
    }

    #[test]
    fn it_resets_value() {
        let mut div = Div::default();
        div.step(0xFF);
        div.step(0x01);
        div.reset_value();

        assert_eq!(div.read(), 0);
    }

    #[test]
    fn it_resets_value_but_keeps_counting_cycles_internally() {
        let mut div = Div::default();
        div.step(0xFF);
        div.step(0xFF);
        div.reset_value();
        div.step(0xFF);

        assert_eq!(div.read(), 1);
    }
}
