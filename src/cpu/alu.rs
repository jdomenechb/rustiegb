use super::registers::CPURegisters;
use std::num::Wrapping;

#[derive(Debug)]
pub struct ALU {}

impl ALU {
    pub fn add_n(&self, registers: &mut CPURegisters, a: u8, b: u8) -> u8 {
        registers.set_flag_n(false);

        let half_carry : bool = ((a & 0xf) + (b & 0xf)) & 0x10 == 0x10;
        registers.set_flag_h(half_carry);

        let carry: bool = (a as u16 + b as u16) & 0x100 == 0x100;
        registers.set_flag_c(carry);

        let value = Wrapping(a);
        let to_add = Wrapping(b);

        let value :u8 = (value + to_add).0;

        let zero :bool = value == 0;
        registers.set_flag_z(zero);

        return value;
    }

    pub fn inc_n(&self, registers: &mut CPURegisters, a: u8) -> u8 {
        let b = 1;

        registers.set_flag_n(false);

        let half_carry : bool = ((a & 0xf) + (b & 0xf)) & 0x10 == 0x10;
        registers.set_flag_h(half_carry);

        let value = Wrapping(a);
        let to_add = Wrapping(b);

        let value :u8 = (value + to_add).0;

        let zero :bool = value == 0;
        registers.set_flag_z(zero);

        return value;
    }

    pub fn sub_n(&self, registers: &mut CPURegisters, a: u8, b: u8) -> u8 {
        registers.set_flag_n(true);

        let half_carry : bool = b > a & 0x0f;
        registers.set_flag_h(half_carry);

        let carry: bool = b > a;
        registers.set_flag_c(carry);

        let value = Wrapping(a);
        let to_subtract = Wrapping(b);

        let value = (value - to_subtract).0;

        let zero :bool = value == 0;
        registers.set_flag_z(zero);

        return value;
    }

    pub fn dec_n(&self, registers: &mut CPURegisters, a: u8,) -> u8 {
        let b = 1;
        registers.set_flag_n(true);

        let half_carry : bool = b > a & 0x0f;
        registers.set_flag_h(half_carry);

        let value = Wrapping(a);
        let to_subtract = Wrapping(b);

        let value = (value - to_subtract).0;

        let zero :bool = value == 0;
        registers.set_flag_z(zero);

        return value;
    }

    pub fn sbc_n(&self, registers: &mut CPURegisters, a: u8, b: u8) -> u8 {
        self.sub_n(registers, a, b - 1)
    }

    pub fn or_n(&self, registers: &mut CPURegisters, a: u8, b: u8) -> u8 {
        let result :u8 = a | b;
        let zero :bool = result == 0;

        registers.set_flag_h(false);
        registers.set_flag_n(false);
        registers.set_flag_z(zero);
        registers.set_flag_c(false);

        return result;
    }

    pub fn and_n(&self, registers: &mut CPURegisters, a: u8, b: u8) -> u8 {
        let result :u8 = a & b;
        let zero :bool = result == 0;

        registers.set_flag_h(false);
        registers.set_flag_n(false);
        registers.set_flag_z(zero);
        registers.set_flag_c(false);

        return result;
    }

    pub fn cp_n(&self, registers: &mut CPURegisters, a: u8, b: u8) {
        registers.set_flag_n(true);

        let zero :bool  = a == b;
        registers.set_flag_z(zero);

        let carry: bool = a < b;
        registers.set_flag_c(carry);

        let half_carry : bool = b > a & 0x0f;
        registers.set_flag_h(half_carry);
    }

    pub fn swap_n(&self, registers: &mut CPURegisters, value: u8) -> u8 {
        registers.set_flag_n(false);
        registers.set_flag_c(false);
        registers.set_flag_h(false);

        let new_low = (value >> 2) & 0b11;
        let new_high = (value << 2) & 0b1100;

        let new_value = new_low | new_high;

        registers.set_flag_z(new_value == 0);

        new_value
    }


    // --- 16 bit ----------------------------------------------------------------------------------

    pub fn add_nn(&self, registers: &mut CPURegisters, a: u16, b: u16) -> u16 {
        registers.set_flag_n(false);

        let half_carry : bool = ((a & 0b11111111111) + (b & 0b11111111111)) & 0b10000000000 == 0b10000000000;
        registers.set_flag_h(half_carry);

        let carry: bool = (a as u32 + b as u32) & 0b10000000000000000 == 0b10000000000000000;
        registers.set_flag_c(carry);

        let value = Wrapping(a);
        let to_add = Wrapping(b);

        let value = (value + to_add).0;

        return value;
    }

    pub fn add_nn_signed(&self, registers: &mut CPURegisters, a: u16, b: i16) -> u16 {
        if b >= 0 {
            self.add_nn(registers, a, b as u16)
        } else {
            self.sub_nn(registers, a, (b * -1) as u16)
        }
    }

    pub fn sub_nn(&self, registers: &mut CPURegisters, a: u16, b: u16) -> u16 {
        registers.set_flag_n(true);

        let half_carry : bool = b > a & 0x0f;
        registers.set_flag_h(half_carry);

        let carry: bool = b > a;
        registers.set_flag_c(carry);

        let value = Wrapping(a);
        let to_subtract = Wrapping(b);

        let value = (value - to_subtract).0;

        let zero :bool = value == 0;
        registers.set_flag_z(zero);

        return value;
    }

    pub fn inc_nn(&self, value: u16) -> u16 {
        let value = Wrapping(value);
        let to_add = Wrapping(1);

        let value :u16 = (value + to_add).0;

        return value;
    }

    pub fn dec_nn(&self, value: u16) -> u16 {
        let value = Wrapping(value);
        let to_add = Wrapping(1);

        let value :u16 = (value - to_add).0;

        return value;
    }
}