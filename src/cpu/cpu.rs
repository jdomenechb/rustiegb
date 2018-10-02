use super::registers::CPURegisters;
use super::alu::ALU;
use ::memory::memory::Memory;

#[derive(Debug)]
pub struct CPU {
    pub registers: CPURegisters,
    alu: ALU,
    trace: bool
}

impl CPU {
    pub fn new() -> CPU {
        return CPU {
            registers: CPURegisters::new(),
            alu: ALU {},
            trace: false
        }
    }

    // --- INSTRUCTIONS ---------------------------------------------------------------------------------------------------------------------

    /**
     * NOP instruction.
     */
    pub fn nop(&mut self) {
        println!("NOP");

        self.registers.pc += 1;
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

        self.registers.pc += 1;
    }

    /**
     * Decrease register C.
     */
    pub fn dec_c(&mut self) {
        println!("DEC C");

        let value = self.registers.c;
        let value = self.alu.sub_n(&mut self.registers, value, 1);
        self.registers.c = value;

        self.registers.pc += 1;
    }

    /**
     * Decrease register D.
     */
    pub fn dec_d(&mut self) {
        println!("DEC D");

        let value = self.registers.d;
        let value = self.alu.sub_n(&mut self.registers, value, 1);
        self.registers.d = value;

        self.registers.pc += 1;
    }

    pub fn inc_hl(&mut self) {
        println!("INC HL");

        let value = self.registers.read_hl();
        self.registers.write_bc(self.alu.inc_nn(value));
        self.registers.pc += 1;
    }

    pub fn inc_bc(&mut self) {
        println!("INC BC");

        let value = self.registers.read_bc();
        self.registers.write_bc(self.alu.inc_nn(value));
        self.registers.pc += 1;
    }

    pub fn inc_a(&mut self) {
        println!("INC A");

        let value :u8 = self.registers.a;
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        self.registers.a = value;
        self.registers.pc += 1;
    }

    pub fn inc_d(&mut self) {
        println!("INC D");

        let value :u8 = self.registers.d;
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        self.registers.d = value;
        self.registers.pc += 1;
    }

    pub fn inc_c(&mut self) {
        println!("INC C");

        let value :u8 = self.registers.c;
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        self.registers.c = value;
        self.registers.pc += 1;
    }

    pub fn adc_a_c(&mut self) {
        let value1 :u8 = self.registers.a;
        let value2 :u8 = self.registers.c + self.registers.is_flag_c() as u8;

        println!("ADC A,C");

        let result :u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.a = result;

        self.registers.pc += 1;
    }

    pub fn add_a_n(&mut self, memory: &Memory) {
        let value1 :u8 = memory.read_8(self.registers.pc + 1);
        let value2 :u8 = self.registers.a;

        println!("ADD A,{:X}", value1);

        let result :u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.a = result;

        self.registers.pc += 2;
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

        self.registers.pc += 1;
    }

    /**
     * XORs register A with register A. Saves result in A. Sets flag Z if result is 0, resets N, H and C. 
     */
    pub fn xor_a(&mut self) {
        println!("XOR A");

        self.registers.a ^= self.registers.a;

        let zero :bool = self.registers.a == 0;
        self.registers.set_flag_z(zero);

        self.registers.set_flag_c(false);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.registers.pc += 1;
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

        self.registers.pc += 1;
    }


    // --- COMPARE INSTRUCTIONS -------------------------------------------------------------------------------------------------------------

    pub fn cp_n(&mut self, memory: &Memory) {       
        let n :u8 = memory.read_8(self.registers.pc + 1);
        let a :u8 = self.registers.a;

        println!("CP {:X}", n);

        self.alu.cp_n(&mut self.registers, a, n);

        self.registers.pc += 2;
    }


    // --- WRITE INSTRUCTIONS ---------------------------------------------------------------------------------------------------------------

    /** 
     * Loads value n to register B. 
     */
    pub fn ld_b_n(&mut self, memory: &Memory) {
        self.registers.b = memory.read_8(self.registers.pc + 1);

        println!("LD B,{:X}", self.registers.b);

        self.registers.pc += 2;
    }

    /** 
     * Loads value n to register C. 
     */
    pub fn ld_c_n(&mut self, memory: &Memory) {
        self.registers.c = memory.read_8(self.registers.pc + 1);

        println!("LD C,{:X}", self.registers.c);

        self.registers.pc += 2;
    }

    /** 
     * Loads register C to register C. 
     */
    pub fn ld_c_c(&mut self) {
        self.registers.c = self.registers.c;

        println!("LD C,C");

        self.registers.pc += 1;
    }

    /** 
     * Loads register C to register C. 
     */
    pub fn ld_a_d(&mut self) {
        self.registers.a = self.registers.d;

        println!("LD A,D");

        self.registers.pc += 1;
    }

    /** 
     * Loads register H to register B. 
     */
    pub fn ld_h_b(&mut self) {
        self.registers.h = self.registers.b;

        println!("LD H,B");

        self.registers.pc += 1;
    }
     

    /** 
     * Loads value n to register E. 
     */
    pub fn ld_e_n(&mut self, memory: &Memory) {
        self.registers.e = memory.read_8(self.registers.pc + 1);

        println!("LD E,{:X}", self.registers.e);

        self.registers.pc += 2;
    }

    /** 
     * Loads value nn to register HL. 
     */
    pub fn ld_hl_nn(&mut self, memory: &Memory) {
        self.registers.l = memory.read_8(self.registers.pc + 1);
        self.registers.h = memory.read_8(self.registers.pc + 2);

        println!("LD HL,{:X}", self.registers.read_hl());

        self.registers.pc += 3;
    }

    /** 
     * Loads value nn to register SP. 
     */
    pub fn ld_sp_nn(&mut self, memory: &Memory) {
        self.registers.sp = memory.read_16(self.registers.pc + 1);

        println!("LD SP,{:X}", self.registers.sp);

        self.registers.pc += 3;
    }

    /** 
     * Loads value n to register A. 
     */
    pub fn ld_a_n(&mut self, memory: &Memory) {
        self.registers.a = memory.read_8(self.registers.pc + 1);

        println!("LD A,{:X}", self.registers.a);

        self.registers.pc += 2;
    }

    /** 
     * Loads value (HL) to register C. 
     */
    pub fn ld_c_mhl(&mut self, memory: &Memory) {
        self.registers.c = memory.read_8(self.registers.read_hl());

        println!("LD C,(HL)");

        self.registers.pc += 1;
    }

    /** 
     * Loads value (HL) to register L. 
     */
    pub fn ld_l_mhl(&mut self, memory: &Memory) {
        self.registers.l = memory.read_8(self.registers.read_hl());

        println!("LD L,(HL)");

        self.registers.pc += 1;
    }

    /** 
     * Loads value (HL) to register H. 
     */
    pub fn ld_h_mhl(&mut self, memory: &Memory) {
        self.registers.h = memory.read_8(self.registers.read_hl());

        println!("LD H,(HL)");

        self.registers.pc += 1;
    }

    /** 
     * Loads value (HL) to register A. 
     */
    pub fn ld_a_mhl(&mut self, memory: &Memory) {
        self.registers.a = memory.read_8(self.registers.read_hl());

        println!("LD A,(HL)");

        self.registers.pc += 1;
    }

    /** 
     * Loads register H to register A. 
     */
    pub fn ld_a_h(&mut self) {
        self.registers.a = self.registers.h;

        println!("LD A,H");

        self.registers.pc += 1;
    }

    /** 
     * Loads register L to register A. 
     */
    pub fn ld_a_l(&mut self) {
        self.registers.a = self.registers.l;

        println!("LD A,L");

        self.registers.pc += 1;
    }

    /** 
     * Loads register B to register A. 
     */
    pub fn ld_a_b(&mut self) {
        self.registers.a = self.registers.b;

        println!("LD A,B");

        self.registers.pc += 1;
    }

    /** 
     * Writes value from register A to memory address $FF00 + n. 
     */
    pub fn ldh_n_a(&mut self, memory: &mut Memory) {
        let to_sum: u16 = memory.read_8(self.registers.pc + 1) as u16;

        println!("LDH ($FF00 + {:X}),A", to_sum);

        let mem_addr: u16 = 0xFF00 + to_sum;

        memory.write_8(mem_addr, self.registers.a);

        self.registers.pc += 2;
    }

    /** 
     * Writes value from memory address $FF00 + n to register A. 
     */
    pub fn ldh_a_n(&mut self, memory: &mut Memory) {
        let to_sum: u16 = memory.read_8(self.registers.pc + 1) as u16;

        println!("LDH ($FF00 + {:X}),A", to_sum);

        let mem_addr: u16 = 0xFF00 + to_sum;
        self.registers.a = memory.read_8(mem_addr);

        self.registers.pc += 2;
    }

    /** 
     * Writes value from register A to memory address nn. 
     */
    pub fn ld_nn_a(&mut self, memory: &mut Memory) {
        let mem_addr:u16 = memory.read_16(self.registers.pc + 1);

        println!("LD ({:X}),A", mem_addr);

        memory.write_8(mem_addr, self.registers.a);

        self.registers.pc += 3;
    }

    /** 
     * Writes value from register A to memory address contained in HL. 
     */
    pub fn ldd_mhl_a(&mut self, memory: &mut Memory) {
        println!("LDD (HL),A");

        memory.write_8(self.registers.read_hl(), self.registers.a);

        let value :u16 = self.registers.read_hl() - 1;
        self.registers.write_hl(value);
    
        self.registers.pc += 1;
    }

    pub fn ldi_a_mhl(&mut self, memory: &Memory) {
        println!("LDI A, (HL)");

        let value: u8 = memory.read_8(self.registers.read_hl());
        self.registers.a = value;

        self.registers.pc += 1;
    }

    /** 
     * Loads value nn to register SP. 
     */
    pub fn ld_bc_nn(&mut self, memory: &Memory) {
        let value: u16 = memory.read_16(self.registers.pc + 1);
        self.registers.write_bc(value);

        println!("LD BC,{:X}", self.registers.read_bc());

        self.registers.pc += 3;
    }


    // --- JUMP INSTRUCTIONS ----------------------------------------------------------------------------------------------------------------

    /**
     * Jumps to the current PC + n
     */
    pub fn jr_n(&mut self, memory: &Memory) {
        let to_sum = memory.read_8_signed(self.registers.pc + 1) + 2;

        self.registers.pc += to_sum as u16;

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
        }
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
        }
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
        }
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
        }
    }

    /**
     * Jumps to the 16 bit address given. 
     */
    pub fn jp_nn(&mut self, memory: &Memory) {
        self.registers.pc = memory.read_16(self.registers.pc + 1);

        println!("JP {:X}", self.registers.pc);
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
    }

    /**
     * Pop two bytes from stack & jump to that address.
     */
    pub fn ret(&mut self, memory: &mut Memory) {
        println!("RET");
        self.registers.pc = self.pop_nn(memory);
    }


    // --- RESTART INSTRUCTIONS ------------------------------------------------------------------------------------------------------------

    pub fn rst_18(&mut self, memory: &mut Memory) {     
        println!("RST $18");
        let current_addr :u16 = self.registers.pc;
        self.push_nn(memory, current_addr);

        self.registers.pc = 0x18;
    }

    pub fn rst_38(&mut self, memory: &mut Memory) {
        println!("RST $38");
        let current_addr :u16 = self.registers.pc;
        self.push_nn(memory, current_addr);

        self.registers.pc = 0x38;
    }


    // --- STACK INSTRUCTIONS --------------------------------------------------------------------------------------------------------------

    /**
     * Push HL into stack.
     */
    pub fn push_hl(&mut self, memory : &mut Memory) {
        println!("PUSH HL");
        let reg: u16 = self.registers.read_hl();
        self.push_nn(memory, reg);
        self.registers.pc += 1;
    }

    /**
     * Push BC into stack.
     */
    pub fn push_bc(&mut self, memory : &mut Memory) {
        println!("PUSH BC");
        let reg: u16 = self.registers.read_bc();
        self.push_nn(memory, reg);
        self.registers.pc += 1;
    }

    /**
     * Push AF into stack.
     */
    pub fn push_af(&mut self, memory : &mut Memory) {
        println!("PUSH AF");
        let reg: u16 = self.registers.read_af();
        self.push_nn(memory, reg);
        self.registers.pc += 1;
    }

    /**
     * Pops stack to AF.
     */
    pub fn pop_af(&mut self, memory : &mut Memory) {
        println!("POP AF");

        let popped: u16 = self.pop_nn(memory);
        self.registers.write_af(popped);
        
        self.registers.pc += 1;
    }

    /**
     * Pops stack to HL.
     */
    pub fn pop_hl(&mut self, memory : &mut Memory) {
        println!("POP HL");

        let popped: u16 = self.pop_nn(memory);
        self.registers.write_hl(popped);
        
        self.registers.pc += 1;
    }


    // --- OTHER INSTRUCTIONS --------------------------------------------------------------------------------------------------------------

    /**
     * Disables interrupts
     */
    pub fn di(&mut self) {
        println!("DI");
        
        // TODO

        self.registers.pc += 1;
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