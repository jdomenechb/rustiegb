use crate::cpu::registers::{ByteRegister, CpuRegisters};
use crate::{Byte, Word};

pub struct Alu {}

impl Alu {
    pub fn add_n(&self, registers: &mut CpuRegisters, a: Byte, b: Byte, carry: bool) -> Byte {
        registers.set_flag_n(false);
        registers.set_flag_h(((a & 0xf) + (b & 0xf) + (carry as Byte)) & 0x10 == 0x10);
        registers.set_flag_c((a as Word + b as Word + carry as Word) & 0x100 == 0x100);

        let value = a.wrapping_add(b).wrapping_add(carry as Byte);

        registers.set_flag_z(value == 0);

        value
    }

    pub fn inc_n(&self, registers: &mut CpuRegisters, a: Byte) -> Byte {
        let b = 1;

        registers.set_flag_n(false);
        registers.set_flag_h(((a & 0xf) + (b & 0xf)) & 0x10 == 0x10);

        let value = a.wrapping_add(b);

        registers.set_flag_z(value == 0);

        value
    }

    pub fn sub_n(&self, registers: &mut CpuRegisters, a: Byte, b: Byte, carry: bool) -> Byte {
        registers.set_flag_n(true);
        registers.set_flag_h((a & 0x0f) < ((b & 0x0f) + (carry as Byte)));
        registers.set_flag_c((a as Word) < ((b as Word) + (carry as Word)));

        let value = a.wrapping_sub(b).wrapping_sub(carry as Byte);

        registers.set_flag_z(value == 0);

        value
    }

    pub fn dec_n(&self, registers: &mut CpuRegisters, a: Byte) -> Byte {
        let b = 1;

        registers.set_flag_n(true);
        registers.set_flag_h(b > a & 0x0f);

        let value = a.wrapping_sub(b);

        registers.set_flag_z(value == 0);

        value
    }

    pub fn or_n(&self, registers: &mut CpuRegisters, a: Byte, b: Byte) -> Byte {
        let result = a | b;

        registers.set_flag_z(result == 0);
        registers.set_flag_n(false);
        registers.set_flag_h(false);
        registers.set_flag_c(false);

        result
    }

    pub fn and_n(&self, registers: &mut CpuRegisters, a: Byte, b: Byte) -> Byte {
        let result = a & b;

        registers.set_flag_z(result == 0);
        registers.set_flag_n(false);
        registers.set_flag_h(true);
        registers.set_flag_c(false);

        result
    }

    pub fn cp_n(&self, registers: &mut CpuRegisters, b: Byte) {
        let a = registers.read_byte(&ByteRegister::A);

        registers.set_flag_z(a == b);
        registers.set_flag_n(true);
        registers.set_flag_h((a & 0x0f) < (b & 0x0f));
        registers.set_flag_c(a < b);
    }

    pub fn swap_n(&self, registers: &mut CpuRegisters, value: Byte) -> Byte {
        registers.set_flag_n(false);
        registers.set_flag_c(false);
        registers.set_flag_h(false);
        registers.set_flag_z(value == 0);

        let new_low = (value >> 4) & 0x0F;
        let new_high = (value << 4) & 0xF0;

        new_low | new_high
    }

    // --- 16 bit ----------------------------------------------------------------------------------

    pub fn add_nn(&self, registers: &mut CpuRegisters, a: Word, b: Word) -> Word {
        registers.set_flag_n(false);

        let half_carry = ((a & 0xFFF) + (b & 0xFFF)) & 0x1000 == 0x1000;
        registers.set_flag_h(half_carry);

        let carry = (a as u32 + b as u32) & 0x10000 == 0x10000;
        registers.set_flag_c(carry);

        a.wrapping_add(b)
    }

    pub fn add_nn_signed(&self, registers: &mut CpuRegisters, a: Word, b: i16) -> Word {
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(0, 0, false, 0, false, false, true  ; "0 without carry")]
    #[test_case(0, 0, true, 1, false, false, false  ; "0 with carry")]
    #[test_case(4, 5, false, 9, false, false, false  ; "normal without carry")]
    #[test_case(4, 5, true, 10, false, false, false  ; "normal with carry")]
    #[test_case(10, 6, false, 16, true, false, false  ; "flag h")]
    #[test_case(0xFF, 1, false, 0, true, true, true  ; "upper limit: flag c,z,h")]
    #[test_case(0xFF, 5, false, 4, true, true, false  ; "upper limit: flag c,h but not z")]
    fn test_add_n(
        a: Byte,
        b: Byte,
        carry: bool,
        expected: Byte,
        expected_h: bool,
        expected_c: bool,
        expected_z: bool,
    ) {
        let mut registers = CpuRegisters::default();
        let alu = Alu {};

        let result = alu.add_n(&mut registers, a, b, carry);

        assert_eq!(result, expected);
        assert!(!registers.is_flag_n());
        assert_eq!(registers.is_flag_h(), expected_h);
        assert_eq!(registers.is_flag_c(), expected_c);
        assert_eq!(registers.is_flag_z(), expected_z);
    }
}
