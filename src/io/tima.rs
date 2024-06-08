use crate::Byte;

#[derive(Default)]
pub struct Tima {
    pub value: Byte,
    remaining_timer_cycles: u32,
}

impl Tima {
    pub fn reset_cycles(&mut self) {
        self.remaining_timer_cycles = 0;
    }

    pub fn step(&mut self, last_instruction_cycles: u8, divider: u32) -> bool {
        self.remaining_timer_cycles += last_instruction_cycles as u32;

        let to_add = (self.remaining_timer_cycles / divider) as u8;
        self.remaining_timer_cycles %= divider;

        let addition_result = self.value.overflowing_add(to_add);

        self.value = addition_result.0;

        addition_result.1
    }
}
