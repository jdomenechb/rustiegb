use crate::{Byte, Word};

pub trait FrequencyUpdatable {
    fn set_frequency(&mut self, frequency: Word);
    fn get_frequency(&self) -> Word;

    fn set_high_part_from_register(&mut self, register: Byte) {
        let frequency_low = self.get_frequency() & 0xFF;
        let frequency_hi = ((register & 0b111) as Word) << 8;

        self.set_frequency(frequency_hi | (frequency_low as Word));
    }

    fn set_low_part_from_register(&mut self, register: Byte) {
        let frequency_low = register;
        let frequency_hi = self.get_frequency() & 0x700;

        self.set_frequency(frequency_hi | (frequency_low as Word));
    }
}

pub trait FrequencyRegisterUpdatable: FrequencyUpdatable {
    fn trigger_frequency_register_update(&mut self, register: Byte) {
        self.set_low_part_from_register(register);
    }
}
