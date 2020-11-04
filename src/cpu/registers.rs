use std::string::ToString;
use strum_macros;
use crate::math::{two_u8_to_u16, u16_to_two_u8};

#[derive(strum_macros::ToString)]
pub enum ByteRegister {
    A, B, C, D, E, F, H, L
}

#[derive(strum_macros::ToString)]
pub enum WordRegister {
    AF, BC, DE, HL, PC, SP
}

#[derive(Debug)]
pub struct CPURegisters {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub flags: u8
}

impl CPURegisters {
    pub fn new(bootstrap: bool) -> CPURegisters {
        return CPURegisters {
            a: 0x01,
            b: 0x0,
            c: 0x13,
            d: 0x0,
            e: 0xd8,
            f: 0xb0,
            h: 0x01,
            l: 0x4d,
            pc: if bootstrap { 0x0001 } else { 0x0100 },
            sp: 0xFFFE,
            flags: 0x0,
        };
    }

    pub fn read_byte(&self, register: ByteRegister) -> u8 {
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

    pub fn read_word(&self, register: WordRegister) -> u16 {
        match register {
            WordRegister::AF => two_u8_to_u16(self.a, self.f),
            WordRegister::BC => two_u8_to_u16(self.b, self.c),
            WordRegister::DE => two_u8_to_u16(self.d, self.e),
            WordRegister::HL => two_u8_to_u16(self.h, self.l),
            WordRegister::PC => self.pc,
            WordRegister::SP => self.sp,
        }
    }

    pub fn write_byte(&mut self, register: ByteRegister, value: u8) {
        match register {
            ByteRegister::A => self.a = value,
            ByteRegister::B => self.b = value,
            ByteRegister::C => self.c = value,
            ByteRegister::D => self.d = value,
            ByteRegister::E => self.e = value,
            ByteRegister::F => self.f = value,
            ByteRegister::H => self.h = value,
            ByteRegister::L => self.l = value,
        }
    }

    pub fn write_word(&mut self, register: WordRegister, value: u16)  {
        let parts: (u8, u8) = u16_to_two_u8(value);

        match register {
            WordRegister::AF => { self.a = parts.0; self.f = parts.1 },
            WordRegister::BC => { self.b = parts.0; self.c = parts.1 },
            WordRegister::DE => { self.d = parts.0; self.e = parts.1 },
            WordRegister::HL => { self.h = parts.0; self.l = parts.1 },
            WordRegister::PC => self.pc = value,
            WordRegister::SP => self.sp = value,
        }
    }

    pub fn read_bc(&self) -> u16 {
        return crate::math::two_u8_to_u16(self.b, self.c);
    }

    pub fn write_bc(&mut self, value : u16) {
        let parts: (u8, u8) = crate::math::u16_to_two_u8(value);
        self.b = parts.0;
        self.c = parts.1;
    }

    /**
     * Reads combination of register D and register E.
     */
    pub fn read_de(&self) -> u16 {
        return crate::math::two_u8_to_u16(self.d, self.e);
    }

    pub fn write_de(&mut self, value : u16) {
        let parts: (u8, u8) = crate::math::u16_to_two_u8(value);
        self.d = parts.0;
        self.e = parts.1;
    }

    /**
     * Reads combination of register H and register L.
     */
    pub fn read_hl(&self) -> u16 {
        return crate::math::two_u8_to_u16(self.h, self.l);
    }

    pub fn write_hl(&mut self, value : u16) {
        let parts: (u8, u8) = crate::math::u16_to_two_u8(value);
        self.h = parts.0;
        self.l = parts.1;
    }

    // --- FLAGS ---
    fn set_flag(&mut self, position: u8, value :bool) {
        let mask :u8 = 1 << position; 

        if value  {
            self.flags |= mask;
        } else {
            self.flags &= !mask; 
        }
    }

    pub fn set_flag_z(&mut self, value:bool) {
        self.set_flag(7, value);
    }

    pub fn set_flag_n(&mut self, value:bool) {
        self.set_flag(6, value); 
    }

    pub fn set_flag_h(&mut self, value:bool) {
        self.set_flag(5, value); 
    }

    pub fn set_flag_c(&mut self, value:bool) {
        self.set_flag(4, value); 
    }

    pub fn is_flag_z(&self) -> bool {
        return self.flags & 0b10000000 == 0b10000000;
    }

    pub fn is_flag_c(&self) -> bool {
        return self.flags & 0b00010000 == 0b00010000;
    }

    pub fn is_flag_n(&self) -> bool {
        return self.flags & 0b01000000 == 0b01000000;
    }

    pub fn is_flag_h(&self) -> bool {
        return self.flags & 0b00100000 == 0b00100000;
    }
}