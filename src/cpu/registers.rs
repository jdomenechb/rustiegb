use crate::utils::math::{two_bytes_to_word, word_to_two_bytes};
use crate::{Byte, Word};

#[derive(Copy, Clone)]
pub enum ByteRegister {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

#[derive(Copy, Clone)]
pub enum WordRegister {
    AF,
    BC,
    DE,
    HL,
    PC,
    SP,
}

#[derive(Debug)]
pub struct CpuRegisters {
    pub a: Byte,
    f: Byte,
    b: Byte,
    c: Byte,
    d: Byte,
    e: Byte,
    h: Byte,
    l: Byte,
    pub sp: Word,
    pub pc: Word,
}

impl CpuRegisters {
    pub fn new(bootstrap: bool) -> Self {
        let mut cpu_registers = Self::default();

        if bootstrap {
            cpu_registers.pc = 0x0001;
        }

        cpu_registers
    }

    pub fn read_byte(&self, register: &ByteRegister) -> Byte {
        match register {
            ByteRegister::A => self.a,
            ByteRegister::B => self.b,
            ByteRegister::C => self.c,
            ByteRegister::D => self.d,
            ByteRegister::E => self.e,
            ByteRegister::F => self.f,
            ByteRegister::H => self.h,
            ByteRegister::L => self.l,
        }
    }

    pub fn read_word(&self, register: &WordRegister) -> Word {
        match register {
            WordRegister::AF => two_bytes_to_word(self.a, self.f),
            WordRegister::BC => two_bytes_to_word(self.b, self.c),
            WordRegister::DE => two_bytes_to_word(self.d, self.e),
            WordRegister::HL => two_bytes_to_word(self.h, self.l),
            WordRegister::PC => self.pc,
            WordRegister::SP => self.sp,
        }
    }

    pub fn write_byte(&mut self, register: &ByteRegister, value: Byte) {
        match register {
            ByteRegister::A => self.a = value,
            ByteRegister::B => self.b = value,
            ByteRegister::C => self.c = value,
            ByteRegister::D => self.d = value,
            ByteRegister::E => self.e = value,
            ByteRegister::F => self.f = value & 0xF0,
            ByteRegister::H => self.h = value,
            ByteRegister::L => self.l = value,
        }
    }

    pub fn write_word(&mut self, register: &WordRegister, value: Word) {
        let parts: (Byte, Byte) = word_to_two_bytes(value);

        match register {
            WordRegister::AF => {
                self.a = parts.0;
                self.f = parts.1 & 0xF0
            }
            WordRegister::BC => {
                self.b = parts.0;
                self.c = parts.1
            }
            WordRegister::DE => {
                self.d = parts.0;
                self.e = parts.1
            }
            WordRegister::HL => {
                self.h = parts.0;
                self.l = parts.1
            }
            WordRegister::PC => self.pc = value,
            WordRegister::SP => self.sp = value,
        }
    }

    // --- FLAGS ---
    fn set_flag(&mut self, position: u8, value: bool) {
        let mask = 1 << position;

        if value {
            self.f |= mask;
        } else {
            self.f &= !mask;
        }
    }

    fn read_flag(&self, position: u8) -> bool {
        let mask = 1 << position;

        self.f & mask == mask
    }

    pub fn set_flag_z(&mut self, value: bool) {
        self.set_flag(7, value);
    }

    pub fn set_flag_n(&mut self, value: bool) {
        self.set_flag(6, value);
    }

    pub fn set_flag_h(&mut self, value: bool) {
        self.set_flag(5, value);
    }

    pub fn set_flag_c(&mut self, value: bool) {
        self.set_flag(4, value);
    }

    pub fn is_flag_z(&self) -> bool {
        self.read_flag(7)
    }

    pub fn is_flag_n(&self) -> bool {
        self.read_flag(6)
    }

    pub fn is_flag_h(&self) -> bool {
        self.read_flag(5)
    }

    pub fn is_flag_c(&self) -> bool {
        self.read_flag(4)
    }
}

impl Default for CpuRegisters {
    fn default() -> Self {
        Self {
            a: 0x01,
            b: 0x0,
            c: 0x13,
            d: 0x0,
            e: 0xd8,
            f: 0xb0,
            h: 0x01,
            l: 0x4d,
            pc: 0x0100,
            sp: 0xFFFE,
        }
    }
}
