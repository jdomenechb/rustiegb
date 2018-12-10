use super::registers::CPURegisters;
use super::alu::ALU;
use ::memory::memory::Memory;

#[derive(Debug)]
pub struct CPU {
    pub registers: CPURegisters,
    alu: ALU,
    trace: bool,
    available_cycles: i32,

    pc_to_increment: i8,
    last_instruction_ccycles: i8,

}

impl CPU {
    const AVAILABLE_CCYCLES_PER_FRAME: i32 = 70224;

    pub fn new() -> CPU {
        return CPU {
            registers: CPURegisters::new(),
            alu: ALU {},
            trace: false,
            available_cycles: CPU::AVAILABLE_CCYCLES_PER_FRAME,

            pc_to_increment: -1,
            last_instruction_ccycles: -1
        }
    }

    pub fn reset_available_ccycles(&mut self) {
        self.available_cycles = CPU::AVAILABLE_CCYCLES_PER_FRAME;
    }

    pub fn has_available_ccycles(&self) -> bool {
        return self.available_cycles > 0;
    }

    pub fn step(&mut self, memory: &mut Memory) {
        self.pc_to_increment = -1;
        self.last_instruction_ccycles = -1;

        let instruction: u8 = memory.read_8(self.registers.pc);

        //println!("{:X} ", memory.read_8(0x2A4));

        // if cpu.registers.pc == 0x2A4 {
        //     print!("{:X}: ", instruction);
        //     break;
        // }

        /*if instruction == 0x3E {
            println!("{:#X?}", cpu);
            break;
        }*/

        print!("{:X}: ", self.registers.pc);

        match instruction {
            0x00 => self.nop(),
            0x01 => self.ld_bc_nn(memory),
            0x02 => self.ld_mbc_a(memory),
            0x03 => self.inc_bc(),
            0x05 => self.dec_b(),
            0x06 => self.ld_b_n(&memory),
            0x0C => self.inc_c(),
            0x0D => self.dec_c(),
            0x0E => self.ld_c_n(memory),
            0x11 => self.ld_de_nn(memory),
            0x13 => self.inc_de(),
            0x14 => self.inc_d(),
            0x15 => self.dec_d(),
            0x18 => self.jr_n(memory),
            0x1A => self.ld_a_mde(memory),
            0x1E => self.ld_e_n(memory),
            0x1F => self.rra(),
            0x20 => self.jr_nz_n(memory),
            0x21 => self.ld_hl_nn(memory),
            0x22 => self.ldi_mhl_a(memory),
            0x23 => self.inc_hl(),
            0x24 => self.inc_h(),
            0x25 => self.dec_h(),
            0x26 => self.ld_h_n(&memory),
            0x28 => self.jr_z_n(memory),
            0x2A => self.ldi_a_mhl(memory),
            0x2C => self.inc_l(),
            0x2D => self.dec_l(),
            0x30 => self.jr_nc_n(memory),
            0x31 => self.ld_sp_nn(memory),
            0x32 => self.ldd_mhl_a(memory),
            0x38 => self.jr_c_n(memory),
            0x3C => self.inc_a(),
            0x3E => self.ld_a_n(memory),
            0x46 => self.ld_b_mhl(memory),
            0x47 => self.ld_b_a(),
            0x49 => self.ld_c_c(),
            0x4E => self.ld_c_mhl(memory),
            0x4F => self.ld_c_a(),
            0x56 => self.ld_d_mhl(memory),
            0x57 => self.ld_d_a(),
            0x5F => self.ld_e_a(),
            0x60 => self.ld_h_b(),
            0x66 => self.ld_h_mhl(memory),
            0x6E => self.ld_l_mhl(memory),
            0x72 => self.ld_mhl_d(memory),
            0x77 => self.ld_mhl_a(memory),
            0x78 => self.ld_a_b(),
            0x79 => self.ld_a_c(),
            0x7A => self.ld_a_d(),
            0x7B => self.ld_a_e(),
            0x7C => self.ld_a_h(),
            0x7D => self.ld_a_l(),
            0x7E => self.ld_a_mhl(memory),
            0x89 => self.adc_a_c(),
            0xA9 => self.xor_c(),
            0xAE => self.xor_mhl(memory),
            0xAF => self.xor_a(),
            0xB1 => self.or_c(),
            0xB7 => self.or_a(),
            0xC0 => self.ret_nz(memory),
            0xC1 => self.pop_bc(memory),
            0xC3 => self.jp_nn(memory),
            0xC4 => self.call_nz_nn(memory),
            0xC5 => self.push_bc(memory),
            0xC6 => self.add_a_n(memory),
            0xC9 => self.ret(memory),
            0xCB => self.prefix_cb(memory),
            0xCD => self.call(memory),
            0xD5 => self.push_de(memory),
            0xD6 => self.sub_n(memory),
            0xDF => self.rst_18(memory),
            0xE0 => self.ldh_n_a(memory),
            0xE1 => self.pop_hl(memory),
            0xE5 => self.push_hl(memory),
            0xE6 => self.and_n(memory),
            0xEA => self.ld_nn_a(memory),
            0xEE => self.xor_n(memory),
            0xF0 => self.ldh_a_n(memory),
            0xF1 => self.pop_af(memory),
            0xF3 => self.di(),
            0xF5 => self.push_af(memory),
            0xFA => self.ld_a_nn(memory),
            0xFE => self.cp_n(memory),
            0xFF => self.rst_38(memory),
            _ => {
                println!("Instruction not implemented: {:X}", instruction);
                panic!("{:#X?}", self);
            }
        }

        if self.last_instruction_ccycles < 0 {
            panic!("Instruction does not count ccycles: {:X}", instruction);
        }

        if self.pc_to_increment < 0 {
            panic!("Instruction does not increment PC: {:X}", instruction);
        }

        self.available_cycles -= self.last_instruction_ccycles as i32;
        //println!("Cycles left: {}", self.available_cycles);

        self.registers.pc += self.pc_to_increment as u16;
    }

    // --- INSTRUCTIONS ---------------------------------------------------------------------------------------------------------------------

    /**
     * NOP instruction.
     */
    pub fn nop(&mut self) {
        println!("NOP");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }


    // --- ARITHMETIC INSTRUCTIONS ----------------------------------------------------------------------------------------------------------

    /**
     * Decrease register B.
     */
    pub fn dec_b(&mut self) {
        println!("DEC B");

        let value = self.registers.b;
        let value = self.alu.sub_n(&mut self.registers, value, 1);
        self.registers.b = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Decrease register C.
     */
    pub fn dec_c(&mut self) {
        println!("DEC C");

        let value = self.registers.c;
        let value = self.alu.sub_n(&mut self.registers, value, 1);
        self.registers.c = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Decrease register D.
     */
    pub fn dec_d(&mut self) {
        println!("DEC D");

        let value = self.registers.d;
        let value = self.alu.sub_n(&mut self.registers, value, 1);
        self.registers.d = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Decrease register H.
     */
    pub fn dec_h(&mut self) {
        println!("DEC H");

        let value = self.registers.h;
        let value = self.alu.sub_n(&mut self.registers, value, 1);
        self.registers.h = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Decrease register L.
     */
    pub fn dec_l(&mut self) {
        println!("DEC L");

        let value = self.registers.l;
        let value = self.alu.sub_n(&mut self.registers, value, 1);
        self.registers.l = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn inc_bc(&mut self) {
        println!("INC BC");

        let value = self.registers.read_bc();
        self.registers.write_bc(self.alu.inc_nn(value));

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    pub fn inc_de(&mut self) {
        println!("INC DE");

        let value = self.registers.read_de();
        self.registers.write_de(self.alu.inc_nn(value));

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    pub fn inc_hl(&mut self) {
        println!("INC HL");

        let value = self.registers.read_hl();
        self.registers.write_hl(self.alu.inc_nn(value));

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    pub fn inc_a(&mut self) {
        println!("INC A");

        let value :u8 = self.registers.a;
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        self.registers.a = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn inc_d(&mut self) {
        println!("INC D");

        let value :u8 = self.registers.d;
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        self.registers.d = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn inc_c(&mut self) {
        println!("INC C");

        let value :u8 = self.registers.c;
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        self.registers.c = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn inc_l(&mut self) {
        println!("INC L");

        let value :u8 = self.registers.l;
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        self.registers.l = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn inc_h(&mut self) {
        println!("INC H");

        let value :u8 = self.registers.h;
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        self.registers.h = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn adc_a_c(&mut self) {
        let value1 :u8 = self.registers.a;
        let value2 :u8 = self.registers.c + self.registers.is_flag_c() as u8;

        println!("ADC A,C");

        let result :u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.a = result;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn add_a_n(&mut self, memory: &Memory) {
        let value1 :u8 = memory.read_8(self.registers.pc + 1);
        let value2 :u8 = self.registers.a;

        println!("ADD A,{:X}", value1);

        let result :u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.a = result;

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Substract n from A.
     */
    pub fn sub_n(&mut self, memory: &Memory) {
        println!("SUB n");

        let value = self.registers.a;
        let to_subtract :u8 = memory.read_8(self.registers.pc + 1);
        let value = self.alu.sub_n(&mut self.registers, value, to_subtract);
        self.registers.d = value;

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }


    /**
     * Rotates A right through carry flag.
     */
    pub fn rra(&mut self) {
        println!("RRA");
        let carry = self.registers.is_flag_c();

        let new_carry = self.registers.a & 0x1 == 1;
        let mut new_a = self.registers.a >> 1;

        self.registers.set_flag_c(new_carry);

        if carry {
            new_a |= 0b10000000; 
        } else {
            new_a &= !0b10000000;
        }

        self.registers.a = new_a;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * XORs register A with register A. Saves result in A. Sets flag Z if result is 0, resets N, H and C. 
     */
    pub fn xor_a(&mut self) {
        println!("XOR A");

        self.registers.a = self.registers.a ^ self.registers.a;

        let zero :bool = self.registers.a == 0;
        self.registers.set_flag_z(zero);

        self.registers.set_flag_c(false);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * XORs register C with register A. Saves result in A. Sets flag Z if result is 0, resets N, H and C. 
     */
    pub fn xor_c(&mut self) {
        println!("XOR C");

        self.registers.a = self.registers.c ^ self.registers.a;

        let zero :bool = self.registers.a == 0;
        self.registers.set_flag_z(zero);

        self.registers.set_flag_c(false);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * XORs value 8 bits with register A. Saves result in A. Sets flag Z if result is 0, resets N, H and C. 
     */
    pub fn xor_n(&mut self, memory: &Memory) {
        let value: u8 = memory.read_8(self.registers.pc + 1);
        println!("XOR {:X}", value);

        self.registers.a = value ^ self.registers.a;

        let zero :bool = self.registers.a == 0;
        self.registers.set_flag_z(zero);

        self.registers.set_flag_c(false);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /**
     * XORs value in memory address HL with register A. Saves result in A. Sets flag Z if result is 0, resets N, H and C. 
     */
    pub fn xor_mhl(&mut self, memory: &Memory) {
        println!("XOR (HL)");

        let value = memory.read_8(self.registers.read_hl());
        self.registers.a = value ^ self.registers.a;

        let zero :bool = self.registers.a == 0;
        self.registers.set_flag_z(zero);

        self.registers.set_flag_c(false);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * OR of A with register A, result in A.
     */
    pub fn or_a(&mut self) {
        println!("OR A");

        let value1 : u8 = self.registers.a;
        let value2 : u8 = self.registers.a;

        let result: u8 = self.alu.or_n(&mut self.registers, value1, value2); 

        self.registers.a = result;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * OR of C with register A, result in A.
     */
    pub fn or_c(&mut self) {
        println!("OR C");

        let value1 : u8 = self.registers.c;
        let value2 : u8 = self.registers.a;

        let result: u8 = self.alu.or_n(&mut self.registers, value1, value2); 

        self.registers.a = result;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * AND of n with register A, result in A.
     */
    pub fn and_n(&mut self, memory: &Memory) {
        let value1 :u8 = memory.read_8(self.registers.pc + 1);
        let value2 :u8 = self.registers.a;

        println!("AND {:X}", value1);

        let result: u8 = self.alu.and_n(&mut self.registers, value1, value2); 

        self.registers.a = result;

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }


    // --- COMPARE INSTRUCTIONS -------------------------------------------------------------------------------------------------------------

    pub fn cp_n(&mut self, memory: &Memory) {       
        let n :u8 = memory.read_8(self.registers.pc + 1);
        let a :u8 = self.registers.a;

        println!("CP {:X}", n);

        self.alu.cp_n(&mut self.registers, a, n);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }


    // --- WRITE INSTRUCTIONS ---------------------------------------------------------------------------------------------------------------

    /** 
     * Loads value n to register B. 
     */
    pub fn ld_b_n(&mut self, memory: &Memory) {
        self.registers.b = memory.read_8(self.registers.pc + 1);

        println!("LD B,{:X}", self.registers.b);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value n to register C. 
     */
    pub fn ld_c_n(&mut self, memory: &Memory) {
        self.registers.c = memory.read_8(self.registers.pc + 1);

        println!("LD C,{:X}", self.registers.c);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value n to register H. 
     */
    pub fn ld_h_n(&mut self, memory: &Memory) {
        self.registers.h = memory.read_8(self.registers.pc + 1);

        println!("LD H,{:X}", self.registers.c);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }


    /** 
     * Loads register C to register C. 
     */
    pub fn ld_c_c(&mut self) {
        self.registers.c = self.registers.c;

        println!("LD C,C");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register H to register B. 
     */
    pub fn ld_h_b(&mut self) {
        self.registers.h = self.registers.b;

        println!("LD H,B");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }
     

    /** 
     * Loads value n to register E. 
     */
    pub fn ld_e_n(&mut self, memory: &Memory) {
        self.registers.e = memory.read_8(self.registers.pc + 1);

        println!("LD E,{:X}", self.registers.e);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value nn to register HL. 
     */
    pub fn ld_hl_nn(&mut self, memory: &Memory) {
        self.registers.l = memory.read_8(self.registers.pc + 1);
        self.registers.h = memory.read_8(self.registers.pc + 2);

        println!("LD HL,{:X}", self.registers.read_hl());

        self.pc_to_increment = 3;
        self.last_instruction_ccycles = 12;
    }

    /** 
     * Loads value nn to register SP. 
     */
    pub fn ld_sp_nn(&mut self, memory: &Memory) {
        self.registers.sp = memory.read_16(self.registers.pc + 1);

        println!("LD SP,{:X}", self.registers.sp);

        self.pc_to_increment = 3;
        self.last_instruction_ccycles = 12;
    }

    /** 
     * Loads value n to register A. 
     */
    pub fn ld_a_n(&mut self, memory: &Memory) {
        self.registers.a = memory.read_8(self.registers.pc + 1);

        println!("LD A,{:X}", self.registers.a);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value (HL) to register B. 
     */
    pub fn ld_b_mhl(&mut self, memory: &Memory) {
        self.registers.b = memory.read_8(self.registers.read_hl());

        println!("LD B,(HL)");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value (HL) to register C. 
     */
    pub fn ld_c_mhl(&mut self, memory: &Memory) {
        self.registers.c = memory.read_8(self.registers.read_hl());

        println!("LD C,(HL)");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value (HL) to register D. 
     */
    pub fn ld_d_mhl(&mut self, memory: &Memory) {
        self.registers.d = memory.read_8(self.registers.read_hl());

        println!("LD D,(HL)");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value (HL) to register L. 
     */
    pub fn ld_l_mhl(&mut self, memory: &Memory) {
        self.registers.l = memory.read_8(self.registers.read_hl());

        println!("LD L,(HL)");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value (HL) to register H. 
     */
    pub fn ld_h_mhl(&mut self, memory: &Memory) {
        self.registers.h = memory.read_8(self.registers.read_hl());

        println!("LD H,(HL)");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

     /** 
     * Loads value (DE) to register A. 
     */
    pub fn ld_a_mde(&mut self, memory: &Memory) {
        self.registers.a = memory.read_8(self.registers.read_de());

        println!("LD A,(DE)");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value (HL) to register A. 
     */
    pub fn ld_a_mhl(&mut self, memory: &Memory) {
        self.registers.a = memory.read_8(self.registers.read_hl());

        println!("LD A,(HL)");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads register B to register A. 
     */
    pub fn ld_a_b(&mut self) {
        self.registers.a = self.registers.b;

        println!("LD A,B");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register C to register A. 
     */
    pub fn ld_a_c(&mut self) {
        self.registers.a = self.registers.c;

        println!("LD A,C");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register D to register A 
     */
    pub fn ld_a_d(&mut self) {
        self.registers.a = self.registers.d;

        println!("LD A,D");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register E to register A 
     */
    pub fn ld_a_e(&mut self) {
        self.registers.a = self.registers.e;

        println!("LD A,E");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register H to register A. 
     */
    pub fn ld_a_h(&mut self) {
        self.registers.a = self.registers.h;

        println!("LD A,H");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register L to register A. 
     */
    pub fn ld_a_l(&mut self) {
        self.registers.a = self.registers.l;

        println!("LD A,L");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register A to register B. 
     */
    pub fn ld_b_a(&mut self) {
        self.registers.b = self.registers.a;

        println!("LD B,A");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register A to register C. 
     */
    pub fn ld_c_a(&mut self) {
        self.registers.c = self.registers.a;

        println!("LD C,A");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register A to register D. 
     */
    pub fn ld_d_a(&mut self) {
        self.registers.d = self.registers.a;

        println!("LD D,A");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register A to register E. 
     */
    pub fn ld_e_a(&mut self) {
        self.registers.e = self.registers.a;

        println!("LD E,A");

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Writes value from register A to memory address $FF00 + n. 
     */
    pub fn ldh_n_a(&mut self, memory: &mut Memory) {
        let to_sum: u16 = memory.read_8(self.registers.pc + 1) as u16;

        println!("LDH ($FF00 + {:X}),A", to_sum);

        let mem_addr: u16 = 0xFF00 + to_sum;

        memory.write_8(mem_addr, self.registers.a);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 12;
    }

    /** 
     * Writes value from memory address $FF00 + n to register A. 
     */
    pub fn ldh_a_n(&mut self, memory: &mut Memory) {
        let to_sum: u16 = memory.read_8(self.registers.pc + 1) as u16;

        println!("LDH ($FF00 + {:X}),A", to_sum);

        let mem_addr: u16 = 0xFF00 + to_sum;
        self.registers.a = memory.read_8(mem_addr);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 12;
    }

    /** 
     * Writes value from register A to memory address nn. 
     */
    pub fn ld_nn_a(&mut self, memory: &mut Memory) {
        let mem_addr:u16 = memory.read_16(self.registers.pc + 1);

        println!("LD ({:X}),A", mem_addr);

        memory.write_8(mem_addr, self.registers.a);

        self.pc_to_increment = 3;
        self.last_instruction_ccycles = 16;
    }

    /** 
     * Writes value from memory address nn to register A. 
     */
    pub fn ld_a_nn(&mut self, memory: &mut Memory) {
        let mem_addr:u16 = memory.read_16(self.registers.pc + 1);

        println!("LD A, ({:X})", mem_addr);

        self.registers.a = memory.read_8(mem_addr);

        self.pc_to_increment = 3;
        self.last_instruction_ccycles = 16;
    }

    /** 
     * Writes value from register A to memory address contained in HL and decreases HL. 
     */
    pub fn ldd_mhl_a(&mut self, memory: &mut Memory) {
        println!("LDD (HL),A");

        memory.write_8(self.registers.read_hl(), self.registers.a);

        let value :u16 = self.registers.read_hl();
        self.registers.write_hl(self.alu.dec_nn(value));
    
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Writes value from register A to memory address contained in HL and increases HL. 
     */
    pub fn ldi_mhl_a(&mut self, memory: &mut Memory) {
        println!("LDI (HL),A");

        memory.write_8(self.registers.read_hl(), self.registers.a);

        let value :u16 = self.registers.read_hl();
        self.registers.write_hl(self.alu.inc_nn(value));
    
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Writes value from register A to memory address contained in BC. 
     */
    pub fn ld_mbc_a(&mut self, memory: &mut Memory) {
        println!("LD (BC),A");

        memory.write_8(self.registers.read_bc(), self.registers.a);
    
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Writes value from register A to memory address contained in HL. 
     */
    pub fn ld_mhl_a(&mut self, memory: &mut Memory) {
        println!("LD (HL),A");

        memory.write_8(self.registers.read_hl(), self.registers.a);
    
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Writes value from register D to memory address contained in HL. 
     */
    pub fn ld_mhl_d(&mut self, memory: &mut Memory) {
        println!("LD (HL),D");

        memory.write_8(self.registers.read_hl(), self.registers.d);
    
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    pub fn ldi_a_mhl(&mut self, memory: &Memory) {
        println!("LDI A,(HL)");

        let value: u8 = memory.read_8(self.registers.read_hl());
        self.registers.a = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value nn to register BC. 
     */
    pub fn ld_bc_nn(&mut self, memory: &Memory) {
        let value: u16 = memory.read_16(self.registers.pc + 1);
        self.registers.write_bc(value);

        println!("LD BC,{:X}", self.registers.read_bc());

        self.pc_to_increment = 3;
        self.last_instruction_ccycles = 12;
    }

    /** 
     * Loads value nn to register DE. 
     */
    pub fn ld_de_nn(&mut self, memory: &Memory) {
        let value: u16 = memory.read_16(self.registers.pc + 1);
        self.registers.write_de(value);

        println!("LD DE,{:X}", self.registers.read_de());

        self.pc_to_increment = 3;
        self.last_instruction_ccycles = 12;
    }


    // --- JUMP INSTRUCTIONS ----------------------------------------------------------------------------------------------------------------

    /**
     * Jumps to the current PC + n
     */
    pub fn jr_n(&mut self, memory: &Memory) {
        let to_sum = memory.read_8_signed(self.registers.pc + 1) + 2;

        self.registers.pc += to_sum as u16;

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 12;

        println!("JR {:X}", self.registers.pc);
    }

    /**
     * Jumps to the current PC + n only if the flag Z is not set. Otherwise, continues to the next instruction.
     */
    pub fn jr_nz_n(&mut self, memory: &Memory) {
        let possible_value : i8 = memory.read_8_signed(self.registers.pc + 1);

        println!("JR NZ,{:X}", possible_value);

        self.registers.pc += 2;

        if !self.registers.is_flag_z() {
            self.registers.pc = (self.registers.pc as i16 + possible_value as i16) as u16;
            self.last_instruction_ccycles = 12;
        } else {
            self.last_instruction_ccycles = 8;
        }

        self.pc_to_increment = 0;
    }

    /**
     * Jumps to the current PC + n only if the flag Z is set. Otherwise, continues to the next instruction.
     */
    pub fn jr_z_n(&mut self, memory: &Memory) {
        let possible_value : i8 = memory.read_8_signed(self.registers.pc + 1);

        println!("JR Z,{:X}", possible_value);

        self.registers.pc += 2;

        if self.registers.is_flag_z() {
            self.registers.pc = (self.registers.pc as i16 + possible_value as i16) as u16;
            self.last_instruction_ccycles = 12;
        } else {
            self.last_instruction_ccycles = 8;
        }

        self.pc_to_increment = 0;
    }

    /**
     * Jumps to the current PC + n only if the flag C is set. Otherwise, continues to the next instruction.
     */
    pub fn jr_c_n(&mut self, memory: &Memory) {
        let possible_value : i8 = memory.read_8_signed(self.registers.pc + 1);

        println!("JR C,{:X}", possible_value);

        self.registers.pc += 2;

        if self.registers.is_flag_c() {
            self.registers.pc = (self.registers.pc as i16 + possible_value as i16) as u16;
            self.last_instruction_ccycles = 12;
        } else {
            self.last_instruction_ccycles = 8;
        }

        self.pc_to_increment = 0;
    }

    /**
     * Jumps to the current PC + n only if the flag C is not set. Otherwise, continues to the next instruction.
     */
    pub fn jr_nc_n(&mut self, memory: &Memory) {
        let possible_value : i8 = memory.read_8_signed(self.registers.pc + 1);

        println!("JR NC,{:X}", possible_value);

        self.registers.pc += 2;

        if !self.registers.is_flag_c() {
            self.registers.pc = (self.registers.pc as i16 + possible_value as i16) as u16;
            self.last_instruction_ccycles = 12;
        } else {
            self.last_instruction_ccycles = 8;
        }

        self.pc_to_increment = 0;
    }

    /**
     * Jumps to the 16 bit address given. 
     */
    pub fn jp_nn(&mut self, memory: &Memory) {
        self.registers.pc = memory.read_16(self.registers.pc + 1);

        println!("JP {:X}", self.registers.pc);

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 16;
    }


    // --- FUNC INSTRUCTIONS ---------------------------------------------------------------------------------------------------------------

    /** 
     * Push address of next instruction onto stack and then jump to address nn.
     */
    pub fn call(&mut self, memory: &mut Memory) {
        let next_pc :u16 = self.registers.pc + 3;
        self.push_nn(memory, next_pc);
        
        self.registers.pc = memory.read_16(self.registers.pc + 1);

        println!("CALL {:X}", self.registers.pc);

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 24;
    }

    /** 
     * If flag Z is reset, push address of next instruction onto stack and then jump to address nn.
     */
    pub fn call_nz_nn(&mut self, memory: &mut Memory) {
        println!("CALL NZ,{:X}", self.registers.pc);

        if self.registers.is_flag_z() {
            self.pc_to_increment = 3;
            self.last_instruction_ccycles = 12;
        }

        let next_pc :u16 = self.registers.pc + 3;
        self.push_nn(memory, next_pc);
        
        self.registers.pc = memory.read_16(self.registers.pc + 1);
        
        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 24;
    }

    /**
     * Pop two bytes from stack & jump to that address.
     */
    pub fn ret(&mut self, memory: &mut Memory) {
        println!("RET");

        self.registers.pc = self.pop_nn(memory);

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Pop two bytes from stack & jump to that address if flag Z is not set.
     */
    pub fn ret_nz(&mut self, memory: &mut Memory) {
        println!("RET NZ");

        if !self.registers.is_flag_z() {
            self.registers.pc = self.pop_nn(memory);
            self.last_instruction_ccycles = 20;
        } else {
            self.registers.pc += 1;
            self.last_instruction_ccycles = 8;
        }
        
        self.pc_to_increment = 0;
    }


    // --- RESTART INSTRUCTIONS ------------------------------------------------------------------------------------------------------------

    pub fn rst_18(&mut self, memory: &mut Memory) {     
        println!("RST $18");
        let current_addr :u16 = self.registers.pc;
        self.push_nn(memory, current_addr);

        self.registers.pc = 0x18;

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 16;
    }

    pub fn rst_38(&mut self, memory: &mut Memory) {
        println!("RST $38");
        let current_addr :u16 = self.registers.pc;
        self.push_nn(memory, current_addr);

        self.registers.pc = 0x38;

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 16;
    }


    // --- STACK INSTRUCTIONS --------------------------------------------------------------------------------------------------------------

    /**
     * Push HL into stack.
     */
    pub fn push_hl(&mut self, memory : &mut Memory) {
        println!("PUSH HL");
        let reg: u16 = self.registers.read_hl();
        self.push_nn(memory, reg);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Push BC into stack.
     */
    pub fn push_bc(&mut self, memory : &mut Memory) {
        println!("PUSH BC");
        let reg: u16 = self.registers.read_bc();
        self.push_nn(memory, reg);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Push AF into stack.
     */
    pub fn push_af(&mut self, memory : &mut Memory) {
        println!("PUSH AF");
        let reg: u16 = self.registers.read_af();
        self.push_nn(memory, reg);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Push DE into stack.
     */
    pub fn push_de(&mut self, memory : &mut Memory) {
        println!("PUSH DE");
        let reg: u16 = self.registers.read_de();
        self.push_nn(memory, reg);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Pops stack to AF.
     */
    pub fn pop_af(&mut self, memory : &mut Memory) {
        println!("POP AF");

        let popped: u16 = self.pop_nn(memory);
        self.registers.write_af(popped);
        
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 12;
    }

    /**
     * Pops stack to AF.
     */
    pub fn pop_bc(&mut self, memory : &mut Memory) {
        println!("POP BC");

        let popped: u16 = self.pop_nn(memory);
        self.registers.write_bc(popped);
        
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 12;
    }

    /**
     * Pops stack to HL.
     */
    pub fn pop_hl(&mut self, memory : &mut Memory) {
        println!("POP HL");

        let popped: u16 = self.pop_nn(memory);
        self.registers.write_hl(popped);
        
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 12;
    }

    // --- PREFIX CB -----------------------------------------------------------------------------------------------------------------------

    pub fn prefix_cb(&mut self, memory : &mut Memory)
    {
        let op: u8 = memory.read_8(self.registers.pc + 1);

        print!("CB {:X}: ", op);

        match op {
            0x19 => self.rr_c(),
            0x1A => self.rr_d(),
            0x38 => self.srl_b(),
            _ => {
                println!("CB Instruction not implemented: {:X}", op);
                panic!("{:#X?}", self);
            }
        }
    }

    /** 
     * Rotate right through carry register C.
     */
    pub fn rr_c(&mut self)
    {
        println!("RR C");
        let carry : bool = self.registers.c & 0b1 == 1;
        let msf : u8 = if self.registers.is_flag_c() {0b10000000} else {0};

        self.registers.c = msf | ((self.registers.c >> 1) & 0b01111111);

        let zero :bool = self.registers.c == 0;
        self.registers.set_flag_z(zero);
        self.registers.set_flag_c(carry);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Rotate right through carry register D.
     */
    pub fn rr_d(&mut self)
    {
        println!("RR D");
        let carry : bool = self.registers.d & 0b1 == 1;
        let msf : u8 = if self.registers.is_flag_c() {0b10000000} else {0};

        self.registers.d = msf | ((self.registers.d >> 1) & 0b01111111);

        let zero :bool = self.registers.d == 0;
        self.registers.set_flag_z(zero);
        self.registers.set_flag_c(carry);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Shifts bit 0 of register B to Carry, resets other flags, Z gets updated and MSF is preserved.
     */
    pub fn srl_b(&mut self)
    {
        println!("SRL B");
        let carry : bool = self.registers.b & 0b1 == 1;
        let msf : u8 = self.registers.b & 0b10000000;

        self.registers.b = msf | ((self.registers.b >> 1) & 0b01111111);

        let zero :bool = self.registers.b == 0;

        self.registers.set_flag_z(zero);
        self.registers.set_flag_c(carry);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }


    // --- OTHER INSTRUCTIONS --------------------------------------------------------------------------------------------------------------

    /**
     * Disables interrupts
     */
    pub fn di(&mut self) {
        println!("DI");
        
        // TODO

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }


    // --- PRIVATE METHODS -----------------------------------------------------------------------------------------------------------------
    fn push_nn(&mut self, memory : &mut Memory, value: u16) {
        memory.write_16(self.registers.sp - 2, value);
        self.registers.sp = self.registers.sp - 2;
    }

    fn pop_nn(&mut self, memory : &mut Memory) -> u16 {
        let value = memory.read_16(self.registers.sp);
        self.registers.sp += 2;

        return value;
    }
}