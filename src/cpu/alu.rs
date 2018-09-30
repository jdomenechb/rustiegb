use super::registers::CPURegisters;
use std::num::Wrapping;

#[derive(Debug)]
pub struct ALU {

}

impl ALU {
    pub fn sub_n(&self, registers: &mut CPURegisters, a: u8, b: u8) -> u8 {
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

    pub fn or_n(&self, registers: &mut CPURegisters, a: u8, b: u8) -> u8 {
        let result :u8 = a | b;
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
}