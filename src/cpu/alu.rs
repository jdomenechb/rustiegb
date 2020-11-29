use super::registers::CPURegisters;
use crate::cpu::registers::ByteRegister;
use crate::{Byte, Word};
use std::num::Wrapping;

#[derive(Debug)]
pub struct ALU {}

impl ALU {
    pub fn add_n(&self, registers: &mut CPURegisters, a: Byte, b: Byte) -> Byte {
        registers.set_flag_n(false);
        registers.set_flag_h(((a & 0xf) + (b & 0xf)) & 0x10 == 0x10);
        registers.set_flag_c((a as Word + b as Word) & 0x100 == 0x100);

        let value = a.wrapping_add(b);

        registers.set_flag_z(value == 0);

        value
    }

    pub fn inc_n(&self, registers: &mut CPURegisters, a: Byte) -> Byte {
        let b = 1;

        registers.set_flag_n(false);
        registers.set_flag_h(((a & 0xf) + (b & 0xf)) & 0x10 == 0x10);

        let value = a.wrapping_add(b);

        registers.set_flag_z(value == 0);

        value
    }

    pub fn sub_n(&self, registers: &mut CPURegisters, a: Byte, b: Byte) -> Byte {
        registers.set_flag_n(true);
        registers.set_flag_h(b > a & 0x0f);
        registers.set_flag_c(b > a);

        let value = a.wrapping_sub(b);

        registers.set_flag_z(value == 0);

        value
    }

    pub fn dec_n(&self, registers: &mut CPURegisters, a: Byte) -> Byte {
        let b = 1;

        registers.set_flag_n(true);
        registers.set_flag_h(b > a & 0x0f);

        let value = a.wrapping_sub(b);

        registers.set_flag_z(value == 0);

        value
    }

    pub fn or_n(&self, registers: &mut CPURegisters, a: Byte, b: Byte) -> Byte {
        let result = a | b;

        registers.set_flag_z(result == 0);
        registers.set_flag_n(false);
        registers.set_flag_h(false);
        registers.set_flag_c(false);

        result
    }

    pub fn and_n(&self, registers: &mut CPURegisters, a: Byte, b: Byte) -> Byte {
        let result = a & b;

        registers.set_flag_z(result == 0);
        registers.set_flag_n(false);
        registers.set_flag_h(true);
        registers.set_flag_c(false);

        result
    }

    pub fn cp_n(&self, registers: &mut CPURegisters, b: Byte) {
        let a = registers.read_byte(&ByteRegister::A);

        registers.set_flag_z(a == b);
        registers.set_flag_n(true);
        registers.set_flag_h(b > a & 0x0f);
        registers.set_flag_c(a < b);
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

        let half_carry =
            ((a & 0b11111111111) + (b & 0b11111111111)) & 0b10000000000 == 0b10000000000;
        registers.set_flag_h(half_carry);

        let carry = (a as u32 + b as u32) & 0b10000000000000000 == 0b10000000000000000;
        registers.set_flag_c(carry);

        a.wrapping_add(b)
    }

    pub fn add_nn_signed(&self, registers: &mut CPURegisters, a: Word, b: i16) -> Word {
        let b = b as Word;

        registers.set_flag_z(false);
        registers.set_flag_n(false);
        registers.set_flag_h((a & 0x000f) + (b & 0x000f) > 0x000f);
        registers.set_flag_c((a & 0x00ff) + (b & 0x00ff) > 0x00ff);

        a.wrapping_add(b)
    }

    pub fn inc_nn(&self, value: Word) -> Word {
        value.wrapping_add(1)
    }

    pub fn dec_nn(&self, value: Word) -> Word {
        value.wrapping_sub(1)
    }
}
