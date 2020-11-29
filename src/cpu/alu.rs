use super::registers::CPURegisters;
use crate::cpu::registers::ByteRegister;
use crate::{Byte, Word};
use std::num::Wrapping;

#[derive(Debug)]
pub struct ALU {}

impl ALU {
    pub fn add_n(&self, registers: &mut CPURegisters, a: Byte, b: Byte) -> Byte {
        registers.set_flag_n(false);

        let half_carry: bool = ((a & 0xf) + (b & 0xf)) & 0x10 == 0x10;
        registers.set_flag_h(half_carry);

        let carry: bool = (a as Word + b as Word) & 0x100 == 0x100;
        registers.set_flag_c(carry);

        let value = Wrapping(a);
        let to_add = Wrapping(b);

        let value: Byte = (value + to_add).0;

        let zero: bool = value == 0;
        registers.set_flag_z(zero);

        return value;
    }

    pub fn inc_n(&self, registers: &mut CPURegisters, a: Byte) -> Byte {
        let b = 1;

        registers.set_flag_n(false);

        let half_carry: bool = ((a & 0xf) + (b & 0xf)) & 0x10 == 0x10;
        registers.set_flag_h(half_carry);

        let value = Wrapping(a);
        let to_add = Wrapping(b);

        let value: Byte = (value + to_add).0;

        let zero: bool = value == 0;
        registers.set_flag_z(zero);

        return value;
    }

    pub fn sub_n(&self, registers: &mut CPURegisters, a: Byte, b: Byte) -> Byte {
        registers.set_flag_n(true);

        let half_carry: bool = b > a & 0x0f;
        registers.set_flag_h(half_carry);

        let carry: bool = b > a;
        registers.set_flag_c(carry);

        let value = Wrapping(a);
        let to_subtract = Wrapping(b);

        let value = (value - to_subtract).0;

        let zero: bool = value == 0;
        registers.set_flag_z(zero);

        return value;
    }

    pub fn dec_n(&self, registers: &mut CPURegisters, a: Byte) -> Byte {
        let b = 1;
        registers.set_flag_n(true);

        let half_carry: bool = b > a & 0x0f;
        registers.set_flag_h(half_carry);

        let value = Wrapping(a);
        let to_subtract = Wrapping(b);

        let value = (value - to_subtract).0;

        let zero: bool = value == 0;
        registers.set_flag_z(zero);

        return value;
    }

    pub fn or_n(&self, registers: &mut CPURegisters, a: Byte, b: Byte) -> Byte {
        let result: Byte = a | b;
        let zero: bool = result == 0;

        registers.set_flag_z(zero);
        registers.set_flag_n(false);
        registers.set_flag_h(false);
        registers.set_flag_c(false);

        return result;
    }

    pub fn and_n(&self, registers: &mut CPURegisters, a: Byte, b: Byte) -> Byte {
        let result: Byte = a & b;
        let zero: bool = result == 0;

        registers.set_flag_z(zero);
        registers.set_flag_n(false);
        registers.set_flag_h(true);
        registers.set_flag_c(false);

        return result;
    }

    pub fn cp_n(&self, registers: &mut CPURegisters, b: Byte) {
        let a = registers.read_byte(&ByteRegister::A);
        registers.set_flag_z(a == b);
        registers.set_flag_n(true);

        let half_carry: bool = b > a & 0x0f;
        registers.set_flag_h(half_carry);

        let carry: bool = a < b;
        registers.set_flag_c(carry);
    }

    pub fn swap_n(&self, registers: &mut CPURegisters, value: Byte) -> Byte {
        registers.set_flag_n(false);
        registers.set_flag_c(false);
        registers.set_flag_h(false);
        registers.set_flag_z(value == 0);

        let new_low = (value >> 4) & 0x0F;
        let new_high = (value << 4) & 0xF0;

        new_low | new_high
    }

    // --- 16 bit ----------------------------------------------------------------------------------

    pub fn add_nn(&self, registers: &mut CPURegisters, a: Word, b: Word) -> Word {
        registers.set_flag_n(false);

        let half_carry: bool =
            ((a & 0b11111111111) + (b & 0b11111111111)) & 0b10000000000 == 0b10000000000;
        registers.set_flag_h(half_carry);

        let carry: bool = (a as u32 + b as u32) & 0b10000000000000000 == 0b10000000000000000;
        registers.set_flag_c(carry);

        let value = Wrapping(a);
        let to_add = Wrapping(b);

        let value = (value + to_add).0;

        return value;
    }

    pub fn add_nn_signed(&self, registers: &mut CPURegisters, a: Word, b: i16) -> Word {
        let result;

        if b >= 0 {
            result = self.add_nn(registers, a, b as Word)
        } else {
            result = self.sub_nn(registers, a, (b * -1) as Word)
        }

        registers.set_flag_z(false);
        registers.set_flag_z(false);

        result
    }

    pub fn sub_nn(&self, registers: &mut CPURegisters, a: Word, b: Word) -> Word {
        registers.set_flag_n(true);

        let half_carry: bool = b > a & 0x0f;
        registers.set_flag_h(half_carry);

        let carry: bool = b > a;
        registers.set_flag_c(carry);

        let value = Wrapping(a);
        let to_subtract = Wrapping(b);

        let value = (value - to_subtract).0;

        let zero: bool = value == 0;
        registers.set_flag_z(zero);

        return value;
    }

    pub fn inc_nn(&self, value: Word) -> Word {
        let value = Wrapping(value);
        let to_add = Wrapping(1);

        let value: Word = (value + to_add).0;

        return value;
    }

    pub fn dec_nn(&self, value: Word) -> Word {
        let value = Wrapping(value);
        let to_add = Wrapping(1);

        let value: Word = (value - to_add).0;

        return value;
    }
}
