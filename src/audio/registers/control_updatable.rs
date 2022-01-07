use crate::Byte;

pub trait ControlRegisterUpdatable: ControlUpdatable {
    fn trigger_control_register_update(&mut self, register: Byte);
}

pub trait ControlUpdatable {
    fn calculate_initial_from_register(register: Byte) -> bool {
        register & 0b10000000 == 0b10000000
    }

    fn calculate_use_length_from_register(register: Byte) -> bool {
        register & 0b1000000 == 0b1000000
    }
}
