use crate::{Byte, Word};

pub trait LengthRegisterUpdatable: LengthUpdatable {
    fn trigger_length_register_update(&mut self, register: Byte);
}

pub trait LengthUpdatable {
    fn get_maximum_length() -> Word;

    fn calculate_length_from_register(register: Byte) -> Byte;

    fn set_length(&mut self, length: Byte);

    fn get_length(&mut self) -> Byte;

    fn set_remaining_steps(&mut self, remaining_steps: Word);

    fn refresh_remaining_steps(&mut self) {
        let length = self.get_length();

        self.set_remaining_steps(Self::get_maximum_length() - length as Word);
    }

    fn update_length_from_register(&mut self, register: Byte) {
        let length = Self::calculate_length_from_register(register);

        self.set_length(length);
        self.refresh_remaining_steps();
    }
}
