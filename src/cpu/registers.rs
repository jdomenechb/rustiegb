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
    pub fn new() -> CPURegisters {
        return CPURegisters {
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
            flags: 0x0,
            // a: 0x11,
            // b: 0x0,
            // c: 0x00,
            // d: 0xFF,
            // e: 0x56,
            // f: 0x80,
            // h: 0x00,
            // l: 0x0d,
            // pc: 0x0100,
            // sp: 0xFFFE,
            // flags: 0xB0,
        };
    }

    /**
     * Reads combination of register A and register F.
     */
    pub fn read_af(&self) -> u16 {
        return ::math::two_u8_to_u16(self.a, self.f);
    }

    pub fn write_af(&mut self, value : u16) {
        let parts: (u8, u8) = ::math::u16_to_two_u8(value);
        self.a = parts.0;
        self.f = parts.1;
    }

    /**
     * Reads combination of register B and register C.
     */
    pub fn read_bc(&self) -> u16 {
        return ::math::two_u8_to_u16(self.b, self.c);
    }

    pub fn write_bc(&mut self, value : u16) {
        let parts: (u8, u8) = ::math::u16_to_two_u8(value);
        self.b = parts.0;
        self.c = parts.1;
    }

    /**
     * Reads combination of register D and register E.
     */
    pub fn read_de(&self) -> u16 {
        return ::math::two_u8_to_u16(self.d, self.e);
    }

    /**
     * Reads combination of register H and register L.
     */
    pub fn read_hl(&self) -> u16 {
        return ::math::two_u8_to_u16(self.h, self.l);
    }

    pub fn write_hl(&mut self, value : u16) {
        let parts: (u8, u8) = ::math::u16_to_two_u8(value);
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
}