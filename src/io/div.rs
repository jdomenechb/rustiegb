use crate::Byte;

#[derive(Default, Debug)]
#[readonly::make]
pub struct Div {
    pub value: Byte,
    remaining_div_cycles: u16,
}

impl Div {
    const STEP_CYCLES: u16 = 0x100;
    pub fn step(&mut self, last_instruction_cycles: u8) {
        self.remaining_div_cycles += last_instruction_cycles as u16;

        self.value = self
            .value
            .wrapping_add((self.remaining_div_cycles / Self::STEP_CYCLES) as u8);
        self.remaining_div_cycles %= Self::STEP_CYCLES
    }

    pub fn reset_value(&mut self) {
        self.value = 0;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_does_not_increase_in_its_maximum_value() {
        let mut div = Div::default();
        div.step(0xFF);

        assert_eq!(div.value, 0);
    }
    #[test]
    fn it_increases_with_maximum_value_plus_1() {
        let mut div = Div::default();
        div.step(0xFF);
        div.step(0x01);

        assert_eq!(div.value, 1);
    }

    #[test]
    fn it_keeps_increasing() {
        let mut div = Div::default();
        div.step(0xFF);
        div.step(0xFF);
        div.step(0xFF);

        assert_eq!(div.value, 2);
    }

    #[test]
    fn it_resets_value() {
        let mut div = Div::default();
        div.step(0xFF);
        div.step(0x01);
        div.reset_value();

        assert_eq!(div.value, 0);
    }

    #[test]
    fn it_resets_value_but_keeps_counting_cycles_internally() {
        let mut div = Div::default();
        div.step(0xFF);
        div.step(0xFF);
        div.reset_value();
        div.step(0xFF);

        assert_eq!(div.value, 1);
    }
}
