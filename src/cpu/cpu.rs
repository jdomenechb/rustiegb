use super::registers::CPURegisters;
use super::alu::ALU;
use crate::memory::memory::Memory;
use crate::cpu::registers::{WordRegister, ByteRegister};

#[derive(Debug)]
pub struct CPU {
    pub registers: CPURegisters,
    alu: ALU,
    trace: bool,
    available_cycles: i32,

    pc_to_increment: i8,
    last_instruction_ccycles: i8,
    debug: bool,
    last_executed_instruction: String,
    ime: bool,
    halted: bool,
}

impl CPU {
    const AVAILABLE_CCYCLES_PER_FRAME: i32 = 70221;

    pub fn new(debug: bool, bootstrap: bool) -> CPU {
        return CPU {
            registers: CPURegisters::new(bootstrap),
            alu: ALU {},
            trace: false,
            available_cycles: CPU::AVAILABLE_CCYCLES_PER_FRAME,

            pc_to_increment: -1,
            last_instruction_ccycles: -1,
            debug,
            last_executed_instruction: String::new(),
            ime: false,
            halted: false,
        }
    }

    pub fn reset_available_ccycles(&mut self) {
        self.available_cycles = CPU::AVAILABLE_CCYCLES_PER_FRAME;
    }

    pub fn has_available_ccycles(&self) -> bool {
        return self.available_cycles > 0;
    }

    pub fn get_last_instruction_ccycles(&self) -> u8 {
        debug_assert!(self.last_instruction_ccycles >= 0, "No instruction has been executed yet");

        return self.last_instruction_ccycles as u8;
    }

    pub fn step(&mut self, memory: &mut Memory) {
        self.pc_to_increment = -1;
        self.last_instruction_ccycles = -1;

        if self.registers.pc == 0x100 && memory.has_bootstrap_rom() {
            memory.erase_bootstrap_rom();
        }

        let instruction: u8 = memory.read_8(self.registers.pc);

        let current_pc = self.registers.pc;

        match instruction {
            0x00 => self.nop(),
            0x01 => self.ld_rr_nn(memory, WordRegister::BC),
            0x02 => self.ld_mbc_a(memory),
            0x03 => self.inc_rr(WordRegister::BC),
            0x04 => self.inc_b(),
            0x05 => self.dec_b(),
            0x06 => self.ld_b_n(&memory),
            0x09 => self.add_hl_bc(),
            0x0A => self.ld_a_mbc(memory),
            0x0B => self.dec_rr(WordRegister::BC),
            0x0C => self.inc_c(),
            0x0D => self.dec_c(),
            0x0E => self.ld_c_n(memory),
            0x11 => self.ld_rr_nn(memory, WordRegister::DE),
            0x12 => self.ld_mde_a(memory),
            0x13 => self.inc_rr(WordRegister::DE),
            0x14 => self.inc_d(),
            0x15 => self.dec_d(),
            0x16 => self.ld_d_n(&memory),
            0x17 => self.rla(),
            0x18 => self.jr_n(memory),
            0x19 => self.add_hl_de(),
            0x1A => self.ld_a_mde(memory),
            0x1B => self.dec_rr(WordRegister::DE),
            0x1C => self.inc_e(),
            0x1D => self.dec_e(),
            0x1E => self.ld_e_n(memory),
            0x1F => self.rra(),
            0x20 => self.jr_nz_n(memory),
            0x21 => self.ld_rr_nn(memory, WordRegister::HL),
            0x22 => self.ldi_mhl_a(memory),
            0x23 => self.inc_rr(WordRegister::HL),
            0x24 => self.inc_h(),
            0x25 => self.dec_h(),
            0x26 => self.ld_h_n(&memory),
            0x27 => self.daa(),
            0x28 => self.jr_z_n(memory),
            0x29 => self.add_hl_hl(),
            0x2A => self.ldi_a_mhl(memory),
            0x2B => self.dec_rr(WordRegister::HL),
            0x2C => self.inc_l(),
            0x2D => self.dec_l(),
            0x2E => self.ld_l_n(memory),
            0x2F => self.cpl(),
            0x30 => self.jr_nc_n(memory),
            0x31 => self.ld_rr_nn(memory, WordRegister::SP),
            0x32 => self.ldd_mhl_a(memory),
            0x33 => self.inc_rr(WordRegister::SP),
            0x34 => self.inc_mhl(memory),
            0x35 => self.dec_mhl(memory),
            0x36 => self.ld_mhl_n(memory),
            0x37 => self.scf(),
            0x38 => self.jr_c_n(memory),
            0x3A => self.ldd_a_mhl(memory),
            0x3B => self.dec_rr(WordRegister::SP),
            0x3C => self.inc_a(),
            0x3D => self.dec_a(),
            0x3E => self.ld_a_n(memory),
            0x3F => self.ccf(),
            0x40 => self.ld_b_b(),
            0x46 => self.ld_b_mhl(memory),
            0x47 => self.ld_b_a(),
            0x49 => self.ld_c_c(),
            0x4E => self.ld_c_mhl(memory),
            0x4F => self.ld_c_a(),
            0x54 => self.ld_d_h(),
            0x56 => self.ld_d_mhl(memory),
            0x57 => self.ld_d_a(),
            0x5D => self.ld_e_l(),
            0x5E => self.ld_e_mhl(memory),
            0x5F => self.ld_e_a(),
            0x60 => self.ld_h_b(),
            0x62 => self.ld_h_d(),
            0x66 => self.ld_h_mhl(memory),
            0x67 => self.ld_h_a(),
            0x69 => self.ld_l_c(),
            0x6B => self.ld_l_e(),
            0x6E => self.ld_l_mhl(memory),
            0x6F => self.ld_l_a(),
            0x70 => self.ld_mhl_b(memory),
            0x71 => self.ld_mhl_c(memory),
            0x72 => self.ld_mhl_d(memory),
            0x73 => self.ld_mhl_e(memory),
            0x76 => self.halt(),
            0x77 => self.ld_mhl_a(memory),
            0x78 => self.ld_a_b(),
            0x79 => self.ld_a_c(),
            0x7A => self.ld_a_d(),
            0x7B => self.ld_a_e(),
            0x7C => self.ld_a_h(),
            0x7D => self.ld_a_l(),
            0x7E => self.ld_a_mhl(memory),
            0x81 => self.add_a_c(),
            0x85 => self.add_a_l(),
            0x86 => self.add_a_mhl(memory),
            0x87 => self.add_a_a(),
            0x89 => self.adc_a_c(),
            0x90 => self.sub_b(),
            0x91 => self.sub_c(),
            0xA1 => self.and_c(),
            0xA7 => self.and_a(),
            0xA9 => self.xor_c(),
            0xAE => self.xor_mhl(memory),
            0xAF => self.xor_a(),
            0xB0 => self.or_b(),
            0xB1 => self.or_c(),
            0xB6 => self.or_mhl(memory),
            0xB7 => self.or_a(),
            0xB8 => self.cp_b(),
            0xB9 => self.cp_c(),
            0xBA => self.cp_d(),
            0xBB => self.cp_e(),
            0xBC => self.cp_h(),
            0xBD => self.cp_l(),
            0xBE => self.cp_mhl(memory),
            0xBF => self.cp_a(),
            0xC0 => self.ret_nz(memory),
            0xC1 => self.pop_bc(memory),
            0xC2 => self.jp_nz_nn(memory),
            0xC3 => self.jp_nn(memory),
            0xC4 => self.call_nz_nn(memory),
            0xC5 => self.push_bc(memory),
            0xC6 => self.add_a_n(memory),
            0xC8 => self.ret_z(memory),
            0xC9 => self.ret(memory),
            0xCA => self.jp_z_nn(memory),
            0xCB => self.prefix_cb(memory),
            0xCD => self.call(memory),
            0xCE => self.adc_a_n(memory),
            0xD0 => self.ret_nc(memory),
            0xD1 => self.pop_de(memory),
            0xD5 => self.push_de(memory),
            0xD6 => self.sub_n(memory),
            0xD8 => self.ret_c(memory),
            0xD9 => self.reti(memory),
            0xDF => self.rst_18(memory),
            0xE0 => self.ldh_n_a(memory),
            0xE1 => self.pop_hl(memory),
            0xE2 => self.ld_mc_a(memory),
            0xE5 => self.push_hl(memory),
            0xE6 => self.and_n(memory),
            0xE9 => self.jp_mhl(),
            0xEA => self.ld_nn_a(memory),
            0xEE => self.xor_n(memory),
            0xEF => self.rst_28(memory),
            0xF0 => self.ldh_a_n(memory),
            0xF1 => self.pop_af(memory),
            0xF3 => self.di(),
            0xF5 => self.push_af(memory),
            0xF6 => self.or_n(memory),
            0xF8 => self.ld_hl_sp_n(memory),
            0xF9 => self.ld_sp_hl(),
            0xFA => self.ld_a_nn(memory),
            0xFB => self.ei(),
            0xFE => self.cp_n(memory),
            0xFF => self.rst_38(memory),
            _ => {
                println!("Instruction not implemented: {:X}", instruction);
                panic!("{:#X?}", self);
            }
        }

        debug_assert!(self.last_instruction_ccycles >= 0, "Instruction does not count ccycles: {:X}", instruction);
        debug_assert!(self.pc_to_increment >= 0, "Instruction does not increment PC: {:X}", instruction);

        if self.debug {
            println!("{:X}: {}", current_pc, self.last_executed_instruction);
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
        self.last_executed_instruction = "NOP".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }


    // --- ARITHMETIC INSTRUCTIONS ----------------------------------------------------------------------------------------------------------

    /**
     * Decrease register A.
     */
    pub fn dec_a(&mut self) {
        self.last_executed_instruction = "DEC A".to_string();

        let value = self.registers.a;
        let value = self.alu.sub_n(&mut self.registers, value, 1);
        self.registers.a = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Decrease register B.
     */
    pub fn dec_b(&mut self) {
        self.last_executed_instruction = "DEC B".to_string();

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
        self.last_executed_instruction = "DEC C".to_string();

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
        self.last_executed_instruction = "DEC D".to_string();

        let value = self.registers.d;
        let value = self.alu.sub_n(&mut self.registers, value, 1);
        self.registers.d = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
    * Decrease register E.
    */
    pub fn dec_e(&mut self) {
        self.last_executed_instruction = "DEC E".to_string();

        let value = self.registers.e;
        let value = self.alu.sub_n(&mut self.registers, value, 1);
        self.registers.e = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Decrease register H.
     */
    pub fn dec_h(&mut self) {
        self.last_executed_instruction = "DEC H".to_string();

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
        self.last_executed_instruction = "DEC L".to_string();

        let value = self.registers.l;
        let value = self.alu.sub_n(&mut self.registers, value, 1);
        self.registers.l = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Decrease value of memory address in HL.
     */
    pub fn dec_mhl(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "DEC (HL)".to_string();

        let value = memory.read_8(self.registers.read_hl());
        let value = self.alu.sub_n(&mut self.registers, value, 1);
        memory.write_8(self.registers.read_hl(), value);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 12;
    }

    fn dec_rr(&mut self, register: WordRegister) {
        self.last_executed_instruction = format!("DEC {}", register.to_string().to_uppercase()).to_string();

        let value = self.registers.read_word(&register);
        self.registers.write_word(&register, self.alu.dec_nn(value));

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn inc_rr(&mut self, register: WordRegister) {
        self.last_executed_instruction = format!("INC {}", register.to_string().to_uppercase()).to_string();

        let value = self.registers.read_word(&register);
        self.registers.write_word(&register, self.alu.inc_nn(value));

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    pub fn inc_a(&mut self) {
        self.last_executed_instruction = "INC A".to_string();

        let value :u8 = self.registers.a;
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        self.registers.a = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn inc_d(&mut self) {
        self.last_executed_instruction = "INC D".to_string();

        let value :u8 = self.registers.d;
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        self.registers.d = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn inc_e(&mut self) {
        self.last_executed_instruction = "INC E".to_string();

        let value :u8 = self.registers.e;
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        self.registers.e = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn inc_b(&mut self) {
        self.last_executed_instruction = "INC B".to_string();

        let value :u8 = self.registers.b;
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        self.registers.b = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn inc_c(&mut self) {
        self.last_executed_instruction = "INC C".to_string();

        let value :u8 = self.registers.c;
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        self.registers.c = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }


    pub fn inc_h(&mut self) {
        self.last_executed_instruction = "INC H".to_string();

        let value :u8 = self.registers.h;
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        self.registers.h = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn inc_l(&mut self) {
        self.last_executed_instruction = "INC L".to_string();

        let value :u8 = self.registers.l;
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        self.registers.l = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn inc_mhl(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "INC (HL)".to_string();

        let position = self.registers.read_hl();
        let value :u8 = memory.read_8(position);
        let value :u8 = self.alu.add_n(&mut self.registers, value, 1);
        memory.write_8(position, value);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 12;
    }

    pub fn adc_a_c(&mut self) {
        let value1 :u8 = self.registers.a;
        let value2 :u8 = self.registers.c + self.registers.is_flag_c() as u8;

        self.last_executed_instruction = "ADC A,C".to_string();

        let result :u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.a = result;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn adc_a_n(&mut self, memory : &Memory) {
        let value1 :u8 = self.registers.a;
        let mut value2 :u8 = memory.read_8(self.registers.pc + 1);
        self.last_executed_instruction = format!("ADC A,{:X}", value2).to_string();

        value2 += self.registers.is_flag_c() as u8;

        let result :u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.a = result;

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    pub fn add_a_a(&mut self) {
        let value1 :u8 = self.registers.a;
        let value2 :u8 = self.registers.a;

        self.last_executed_instruction = "ADD A,A".to_string();

        let result :u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.a = result;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn add_a_c(&mut self) {
        let value1 :u8 = self.registers.a;
        let value2 :u8 = self.registers.c;

        self.last_executed_instruction = "ADD A,C".to_string();

        let result :u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.a = result;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn add_a_l(&mut self) {
        let value1 :u8 = self.registers.a;
        let value2 :u8 = self.registers.l;

        self.last_executed_instruction = "ADD A,L".to_string();

        let result :u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.a = result;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    pub fn add_a_n(&mut self, memory: &Memory) {
        let value1 :u8 = memory.read_8(self.registers.pc + 1);
        let value2 :u8 = self.registers.a;

        self.last_executed_instruction = format!("ADD A,{:X}", value1).to_string();

        let result :u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.a = result;

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    pub fn add_a_mhl(&mut self, memory: &Memory) {
        let value1 :u8 = memory.read_8(self.registers.read_hl());
        let value2 :u8 = self.registers.a;

        self.last_executed_instruction = "ADD A,(HL)".to_string();

        let result :u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.a = result;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    pub fn add_hl_bc(&mut self) {
        let value1 = self.registers.read_hl();
        let value2 = self.registers.read_bc();

        self.last_executed_instruction = "ADD HL,BC".to_string();

        let result = self.alu.add_nn(&mut self.registers, value1, value2);
        self.registers.write_hl(result);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    pub fn add_hl_de(&mut self) {
        let value1 = self.registers.read_hl();
        let value2 = self.registers.read_de();

        self.last_executed_instruction = "ADD HL,DE".to_string();

        let result = self.alu.add_nn(&mut self.registers, value1, value2);
        self.registers.write_hl(result);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    pub fn add_hl_hl(&mut self) {
        let value1 = self.registers.read_hl();
        let value2 = self.registers.read_hl();

        self.last_executed_instruction = "ADD HL,HL".to_string();

        let result = self.alu.add_nn(&mut self.registers, value1, value2);
        self.registers.write_hl(result);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Substract n from A.
     */
    pub fn sub_n(&mut self, memory: &Memory) {
        let value = self.registers.a;
        let to_subtract :u8 = memory.read_8(self.registers.pc + 1);

        self.last_executed_instruction = format!("SUB A, {:X}", to_subtract).to_string();

        let value = self.alu.sub_n(&mut self.registers, value, to_subtract);
        self.registers.a = value;

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Substract B from A.
     */
    pub fn sub_b(&mut self) {
        self.last_executed_instruction = "SUB B".to_string();

        let value = self.registers.a;
        let to_subtract :u8 = self.registers.b;
        let value = self.alu.sub_n(&mut self.registers, value, to_subtract);
        self.registers.a = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Substract C from A.
     */
    pub fn sub_c(&mut self) {
        self.last_executed_instruction = "SUB C".to_string();

        let value = self.registers.a;
        let to_subtract :u8 = self.registers.c;
        let value = self.alu.sub_n(&mut self.registers, value, to_subtract);
        self.registers.a = value;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }


    /**
     * Rotates A right through carry flag.
     */
    pub fn rra(&mut self) {
        self.last_executed_instruction = "RRA".to_string();
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
        self.last_executed_instruction = "XOR A".to_string();

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
        self.last_executed_instruction = "XOR C".to_string();

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
        self.last_executed_instruction = format!("XOR {:X}", value).to_string();

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
        self.last_executed_instruction = "XOR (HL)".to_string();

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
        self.last_executed_instruction = "OR A".to_string();

        let value1 : u8 = self.registers.a;
        let value2 : u8 = self.registers.a;

        let result: u8 = self.alu.or_n(&mut self.registers, value1, value2); 

        self.registers.a = result;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * OR of B with register A, result in A.
     */
    pub fn or_b(&mut self) {
        self.last_executed_instruction = "OR B".to_string();

        let value1 : u8 = self.registers.b;
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
        self.last_executed_instruction = "OR C".to_string();

        let value1 : u8 = self.registers.c;
        let value2 : u8 = self.registers.a;

        let result: u8 = self.alu.or_n(&mut self.registers, value1, value2); 

        self.registers.a = result;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * OR of memory address HL with register A, result in A.
     */
    pub fn or_mhl(&mut self, memory: &Memory) {
        self.last_executed_instruction = "OR (HL)".to_string();

        let value1 : u8 = memory.read_8(self.registers.read_hl());
        let value2 : u8 = self.registers.a;

        let result: u8 = self.alu.or_n(&mut self.registers, value1, value2); 

        self.registers.a = result;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /**
     * OR of value with register A, result in A.
     */
    pub fn or_n(&mut self, memory: &Memory) {
        let value1 : u8 = memory.read_8(self.registers.pc + 1);
        let value2 : u8 = self.registers.a;

        self.last_executed_instruction = format!("OR {:X}", value1).to_string();

        let result: u8 = self.alu.or_n(&mut self.registers, value1, value2);

        self.registers.a = result;

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /**
     * AND of register A with register A, result in A.
     */
    pub fn and_a(&mut self) {
        let value1 :u8 = self.registers.a;
        let value2 :u8 = self.registers.a;

        self.last_executed_instruction = "AND A".to_string();

        let result: u8 = self.alu.and_n(&mut self.registers, value1, value2);

        self.registers.a = result;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * AND of register C with register A, result in A.
     */
    pub fn and_c(&mut self) {
        let value1 :u8 = self.registers.c;
        let value2 :u8 = self.registers.a;

        self.last_executed_instruction = "AND C".to_string();

        let result: u8 = self.alu.and_n(&mut self.registers, value1, value2);

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

        self.last_executed_instruction = format!("AND {:X}", value1).to_string();

        let result: u8 = self.alu.and_n(&mut self.registers, value1, value2); 

        self.registers.a = result;

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /**
    * Complement A register
    */
    pub fn cpl(&mut self) {
        self.last_executed_instruction = "CPL".to_string();

        self.registers.set_flag_n(true);
        self.registers.set_flag_h(true);


        self.registers.a = !self.registers.a;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }


    // --- FLAG INSTRUCTIONS -------------------------------------------------------------------------------------------------------------

    /**
    * Set Carry flag
    */
    pub fn scf(&mut self) {
        self.last_executed_instruction = "SCF".to_string();

        self.registers.set_flag_n(false);
        self.registers.set_flag_h(false);
        self.registers.set_flag_c(true);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
    * Complement carry flag
    */
    pub fn ccf(&mut self) {
        self.last_executed_instruction = "CCF".to_string();

        self.registers.set_flag_c(!self.registers.is_flag_c());

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    // --- COMPARE INSTRUCTIONS -------------------------------------------------------------------------------------------------------------

    fn cp_a(&mut self) {
        let n :u8 = self.registers.a;
        let a :u8 = self.registers.a;

        self.last_executed_instruction = "CP A".to_string();

        self.alu.cp_n(&mut self.registers, a, n);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn cp_b(&mut self) {
        let n :u8 = self.registers.b;
        let a :u8 = self.registers.a;

        self.last_executed_instruction = "CP B".to_string();

        self.alu.cp_n(&mut self.registers, a, n);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn cp_c(&mut self) {
        let n :u8 = self.registers.c;
        let a :u8 = self.registers.a;

        self.last_executed_instruction = "CP C".to_string();

        self.alu.cp_n(&mut self.registers, a, n);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn cp_d(&mut self) {
        let n :u8 = self.registers.d;
        let a :u8 = self.registers.a;

        self.last_executed_instruction = "CP D".to_string();

        self.alu.cp_n(&mut self.registers, a, n);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn cp_e(&mut self) {
        let n :u8 = self.registers.e;
        let a :u8 = self.registers.a;

        self.last_executed_instruction = "CP E".to_string();

        self.alu.cp_n(&mut self.registers, a, n);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn cp_h(&mut self) {
        let n :u8 = self.registers.h;
        let a :u8 = self.registers.a;

        self.last_executed_instruction = "CP H".to_string();

        self.alu.cp_n(&mut self.registers, a, n);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn cp_l(&mut self) {
        let n :u8 = self.registers.l;
        let a :u8 = self.registers.a;

        self.last_executed_instruction = "CP L".to_string();

        self.alu.cp_n(&mut self.registers, a, n);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn cp_n(&mut self, memory: &Memory) {
        let n :u8 = memory.read_8(self.registers.pc + 1);
        let a :u8 = self.registers.a;

        self.last_executed_instruction = format!("CP {:X}", n).to_string();

        self.alu.cp_n(&mut self.registers, a, n);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn cp_mhl(&mut self, memory: &Memory) {
        let n :u8 = memory.read_8(self.registers.read_hl());
        let a :u8 = self.registers.a;

        self.last_executed_instruction = "CP (HL)".to_string();

        self.alu.cp_n(&mut self.registers, a, n);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    // --- WRITE INSTRUCTIONS ---------------------------------------------------------------------------------------------------------------

    /** 
     * Loads value n to register B. 
     */
    pub fn ld_b_n(&mut self, memory: &Memory) {
        self.registers.b = memory.read_8(self.registers.pc + 1);

        self.last_executed_instruction = format!("LD B,{:X}", self.registers.b).to_string();

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value n to register C. 
     */
    pub fn ld_c_n(&mut self, memory: &Memory) {
        self.registers.c = memory.read_8(self.registers.pc + 1);

        self.last_executed_instruction = format!("LD C,{:X}", self.registers.c).to_string();

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Loads value n to register D.
     */
    pub fn ld_d_n(&mut self, memory: &Memory) {
        self.registers.d = memory.read_8(self.registers.pc + 1);

        self.last_executed_instruction = format!("LD D,{:X}", self.registers.d).to_string();

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value n to register H. 
     */
    pub fn ld_h_n(&mut self, memory: &Memory) {
        self.registers.h = memory.read_8(self.registers.pc + 1);

        self.last_executed_instruction = format!("LD H,{:X}", self.registers.c).to_string();

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }


    /** 
     * Loads register C to register C. 
     */
    pub fn ld_c_c(&mut self) {
        self.registers.c = self.registers.c;

        self.last_executed_instruction = "LD C,C".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Loads from register A to register H.
     */
    pub fn ld_h_a(&mut self) {
        self.registers.h = self.registers.a;

        self.last_executed_instruction = "LD H,A".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Loads from register B to register H.
     */
    pub fn ld_h_b(&mut self) {
        self.registers.h = self.registers.b;

        self.last_executed_instruction = "LD H,B".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Loads from register D to register H.
     */
    pub fn ld_h_d(&mut self) {
        self.registers.h = self.registers.d;

        self.last_executed_instruction = "LD H,D".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
    * Loads from register A to register L.
    */
    pub fn ld_l_a(&mut self) {
        self.registers.l = self.registers.a;

        self.last_executed_instruction = "LD L,A".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
    * Loads from register C to register L.
    */
    pub fn ld_l_c(&mut self) {
        self.registers.l = self.registers.c;

        self.last_executed_instruction = "LD L,C".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
    * Loads from register E to register L.
    */
    pub fn ld_l_e(&mut self) {
        self.registers.l = self.registers.e;

        self.last_executed_instruction = "LD L,E".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }
     

    /** 
     * Loads value n to register E. 
     */
    pub fn ld_e_n(&mut self, memory: &Memory) {
        self.registers.e = memory.read_8(self.registers.pc + 1);

        self.last_executed_instruction = format!("LD E,{:X}", self.registers.e).to_string();

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Put SP + n effective address into HL.
     */
    pub fn ld_hl_sp_n(&mut self, memory: &Memory) {
        let add1 = self.registers.sp;
        let add2 = memory.read_8(self.registers.pc + 1);

        let new_value = self.alu.add_nn(&mut self.registers, add1, add2 as u16);
        self.registers.write_hl(new_value);

        self.last_executed_instruction = format!("LD HL,SP + {:X}", add2).to_string();

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 12;
    }

    /**
     * Loads register HL to register SP.
     */
    pub fn ld_sp_hl(&mut self) {
        self.registers.sp = self.registers.read_hl();

        self.last_executed_instruction = "LD SP,HL".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value n to register A. 
     */
    pub fn ld_a_n(&mut self, memory: &Memory) {
        self.registers.a = memory.read_8(self.registers.pc + 1);

        self.last_executed_instruction = format!("LD A,{:X}", self.registers.a).to_string();

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Loads value n to register L.
     */
    pub fn ld_l_n(&mut self, memory: &Memory) {
        self.registers.l = memory.read_8(self.registers.pc + 1);

        self.last_executed_instruction = format!("LD L,{:X}", self.registers.l).to_string();

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value (HL) to register B. 
     */
    pub fn ld_b_mhl(&mut self, memory: &Memory) {
        self.registers.b = memory.read_8(self.registers.read_hl());

        self.last_executed_instruction = "LD B,(HL)".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value (HL) to register C. 
     */
    pub fn ld_c_mhl(&mut self, memory: &Memory) {
        self.registers.c = memory.read_8(self.registers.read_hl());

        self.last_executed_instruction = "LD C,(HL)".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value (HL) to register D. 
     */
    pub fn ld_d_mhl(&mut self, memory: &Memory) {
        self.registers.d = memory.read_8(self.registers.read_hl());

        self.last_executed_instruction = "LD D,(HL)".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value (HL) to register L. 
     */
    pub fn ld_l_mhl(&mut self, memory: &Memory) {
        self.registers.l = memory.read_8(self.registers.read_hl());

        self.last_executed_instruction = "LD L,(HL)".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value (HL) to register H. 
     */
    pub fn ld_h_mhl(&mut self, memory: &Memory) {
        self.registers.h = memory.read_8(self.registers.read_hl());

        self.last_executed_instruction = "LD H,(HL)".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Loads value (BC) to register A.
     */
    pub fn ld_a_mbc(&mut self, memory: &Memory) {
        self.registers.a = memory.read_8(self.registers.read_bc());

        self.last_executed_instruction = "LD A,(BC)".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

     /** 
     * Loads value (DE) to register A. 
     */
    pub fn ld_a_mde(&mut self, memory: &Memory) {
        self.registers.a = memory.read_8(self.registers.read_de());

        self.last_executed_instruction = "LD A,(DE)".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads value (HL) to register A. 
     */
    pub fn ld_a_mhl(&mut self, memory: &Memory) {
        self.registers.a = memory.read_8(self.registers.read_hl());

        self.last_executed_instruction = "LD A,(HL)".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Loads register B to register A. 
     */
    pub fn ld_a_b(&mut self) {
        self.registers.a = self.registers.b;

        self.last_executed_instruction = "LD A,B".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register C to register A. 
     */
    pub fn ld_a_c(&mut self) {
        self.registers.a = self.registers.c;

        self.last_executed_instruction = "LD A,C".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register D to register A 
     */
    pub fn ld_a_d(&mut self) {
        self.registers.a = self.registers.d;

        self.last_executed_instruction = "LD A,D".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register E to register A 
     */
    pub fn ld_a_e(&mut self) {
        self.registers.a = self.registers.e;

        self.last_executed_instruction = "LD A,E".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register H to register A. 
     */
    pub fn ld_a_h(&mut self) {
        self.registers.a = self.registers.h;

        self.last_executed_instruction = "LD A,H".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register L to register A. 
     */
    pub fn ld_a_l(&mut self) {
        self.registers.a = self.registers.l;

        self.last_executed_instruction = "LD A,L".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register A to register B. 
     */
    pub fn ld_b_a(&mut self) {
        self.registers.b = self.registers.a;

        self.last_executed_instruction = "LD B,A".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Loads register B to register B.
     */
    pub fn ld_b_b(&mut self) {
        self.registers.b = self.registers.b;

        self.last_executed_instruction = "LD B,B".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register A to register C. 
     */
    pub fn ld_c_a(&mut self) {
        self.registers.c = self.registers.a;

        self.last_executed_instruction = "LD C,A".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register A to register D. 
     */
    pub fn ld_d_a(&mut self) {
        self.registers.d = self.registers.a;

        self.last_executed_instruction = "LD D,A".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Loads register H to register D.
     */
    pub fn ld_d_h(&mut self) {
        self.registers.d = self.registers.h;

        self.last_executed_instruction = "LD D,H".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Loads register A to register E. 
     */
    pub fn ld_e_a(&mut self) {
        self.registers.e = self.registers.a;

        self.last_executed_instruction = "LD E,A".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
    * Loads register L to register E.
    */
    pub fn ld_e_l(&mut self) {
        self.registers.e = self.registers.l;

        self.last_executed_instruction = "LD E,L".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Loads value (HL) to register E.
     */
    pub fn ld_e_mhl(&mut self, memory: &Memory) {
        self.registers.e = memory.read_8(self.registers.read_hl());

        self.last_executed_instruction = "LD E,(HL)".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Writes value from register A to memory address $FF00 + n. 
     */
    pub fn ldh_n_a(&mut self, memory: &mut Memory) {
        let to_sum: u16 = memory.read_8(self.registers.pc + 1) as u16;

        self.last_executed_instruction = format!("LDH ($FF00 + {:X}),A", to_sum).to_string();

        let mem_addr: u16 = 0xFF00 + to_sum;

        memory.write_8(mem_addr, self.registers.a);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 12;
    }

    /**
     * Writes value from register A to memory address $FF00 + C.
     */
    pub fn ld_mc_a(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "LD ($FF00 + C),A".to_string();

        let mem_addr: u16 = 0xFF00 + self.registers.c as u16;
        memory.write_8(mem_addr, self.registers.a);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Writes value from memory address $FF00 + n to register A. 
     */
    pub fn ldh_a_n(&mut self, memory: &mut Memory) {
        let to_sum: u16 = memory.read_8(self.registers.pc + 1) as u16;

        self.last_executed_instruction = format!("LDH A, ($FF00 + {:X})", to_sum).to_string();

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

        self.last_executed_instruction = format!("LD ({:X}),A", mem_addr).to_string();

        memory.write_8(mem_addr, self.registers.a);

        self.pc_to_increment = 3;
        self.last_instruction_ccycles = 16;
    }

    /** 
     * Writes value from memory address nn to register A. 
     */
    pub fn ld_a_nn(&mut self, memory: &mut Memory) {
        let mem_addr:u16 = memory.read_16(self.registers.pc + 1);

        self.last_executed_instruction = format!("LD A, ({:X})", mem_addr).to_string();

        self.registers.a = memory.read_8(mem_addr);

        self.pc_to_increment = 3;
        self.last_instruction_ccycles = 16;
    }

    /** 
     * Writes value from register A to memory address contained in HL and decreases HL. 
     */
    pub fn ldd_mhl_a(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "LDD (HL),A".to_string();

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
        self.last_executed_instruction = "LDI (HL),A".to_string();

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
        self.last_executed_instruction = "LD (BC),A".to_string();

        memory.write_8(self.registers.read_bc(), self.registers.a);
    
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Writes value from register A to memory address contained in DE.
     */
    pub fn ld_mde_a(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "LD (DE),A".to_string();

        memory.write_8(self.registers.read_de(), self.registers.a);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Writes value from register A to memory address contained in HL. 
     */
    pub fn ld_mhl_a(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "LD (HL),A".to_string();

        memory.write_8(self.registers.read_hl(), self.registers.a);
    
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Writes value from register B to memory address contained in HL. 
     */
    pub fn ld_mhl_b(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "LD (HL),B".to_string();

        memory.write_8(self.registers.read_hl(), self.registers.b);
    
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Writes value from register C to memory address contained in HL. 
     */
    pub fn ld_mhl_c(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "LD (HL),C".to_string();

        memory.write_8(self.registers.read_hl(), self.registers.c);
    
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Writes value from register D to memory address contained in HL. 
     */
    pub fn ld_mhl_d(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "LD (HL),D".to_string();

        memory.write_8(self.registers.read_hl(), self.registers.d);
    
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Writes value from register E to memory address contained in HL.
     */
    pub fn ld_mhl_e(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "LD (HL),E".to_string();

        memory.write_8(self.registers.read_hl(), self.registers.e);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /** 
     * Writes 8bit value to memory address contained in HL. 
     */
    pub fn ld_mhl_n(&mut self, memory: &mut Memory) {
        let value = memory.read_8(self.registers.pc + 1);

        self.last_executed_instruction = format!("LD (HL),{:X}", value).to_string();

        memory.write_8(self.registers.read_hl(), value);
    
        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 12;
    }

    pub fn ldi_a_mhl(&mut self, memory: &Memory) {
        self.last_executed_instruction = "LDI A,(HL)".to_string();

        let mut new_value_hl = self.registers.read_hl();
        let value: u8 = memory.read_8(new_value_hl);
        self.registers.a = value;

        new_value_hl = self.alu.inc_nn(new_value_hl);

        self.registers.write_hl(new_value_hl);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    pub fn ldd_a_mhl(&mut self, memory: &Memory) {
        self.last_executed_instruction = "LDD A,(HL)".to_string();

        let mut new_value_hl = self.registers.read_hl();
        let value: u8 = memory.read_8(new_value_hl);
        self.registers.a = value;

        new_value_hl = self.alu.dec_nn(new_value_hl);

        self.registers.write_hl(new_value_hl);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn ld_rr_nn(&mut self, memory: &Memory, register: WordRegister) {
        let value: u16 = memory.read_16(self.registers.pc + 1);
        self.registers.write_word(&register, value);

        self.last_executed_instruction = format!("LD {},{:X}", register.to_string(), self.registers.read_word(&register)).to_string();

        self.pc_to_increment = 3;
        self.last_instruction_ccycles = 12;
    }

    // --- JUMP INSTRUCTIONS ----------------------------------------------------------------------------------------------------------------

    /**
     * Jumps to the current PC + n
     */
    pub fn jr_n(&mut self, memory: &Memory) {
        let to_sum = memory.read_8_signed(self.registers.pc + 1) + 2;

        self.registers.pc = self.registers.pc.overflowing_add(to_sum as u16).0;

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 12;

        self.last_executed_instruction = format!("JR {:X}", self.registers.pc).to_string();
    }

    /**
     * Jumps to the current PC + n only if the flag Z is not set. Otherwise, continues to the next instruction.
     */
    pub fn jr_nz_n(&mut self, memory: &Memory) {
        let possible_value : i8 = memory.read_8_signed(self.registers.pc + 1);

        self.last_executed_instruction = format!("JR NZ,{:X}", possible_value).to_string();

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

        self.last_executed_instruction = format!("JR Z,{:X}", possible_value).to_string();

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

        self.last_executed_instruction = format!("JR C,{:X}", possible_value).to_string();

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

        self.last_executed_instruction = format!("JR NC,{:X}", possible_value).to_string();

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

        self.last_executed_instruction = format!("JP {:X}", self.registers.pc).to_string();

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Jumps to the 16 bit address contained in HL.
     */
    pub fn jp_mhl(&mut self) {
        self.registers.pc = self.registers.read_hl();

        self.last_executed_instruction = "JP (HL)".to_string();

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Jumps to the indicated address only if the flag Z is set. Otherwise, continues to the next instruction.
     */
    pub fn jp_z_nn(&mut self, memory: &Memory) {
        let possible_value = memory.read_16(self.registers.pc + 1);

        self.last_executed_instruction = format!("JP Z,{:X}", possible_value).to_string();

        self.registers.pc += 3;

        if self.registers.is_flag_z() {
            self.registers.pc = possible_value;
            self.last_instruction_ccycles = 16;
        } else {
            self.last_instruction_ccycles = 12;
        }

        self.pc_to_increment = 0;
    }

    /**
     * Jumps to the indicated address only if the flag Z is NOT set. Otherwise, continues to the next instruction.
     */
    pub fn jp_nz_nn(&mut self, memory: &Memory) {
        let possible_value = memory.read_16(self.registers.pc + 1);

        self.last_executed_instruction = format!("JP NZ,{:X}", possible_value).to_string();

        self.registers.pc += 3;

        if !self.registers.is_flag_z() {
            self.registers.pc = possible_value;
            self.last_instruction_ccycles = 16;
        } else {
            self.last_instruction_ccycles = 12;
        }

        self.pc_to_increment = 0;
    }


    // --- FUNC INSTRUCTIONS ---------------------------------------------------------------------------------------------------------------

    /** 
     * Push address of next instruction onto stack and then jump to address nn.
     */
    pub fn call(&mut self, memory: &mut Memory) {
        let next_pc :u16 = self.registers.pc + 3;
        self.push_vv(memory, next_pc);
        
        self.registers.pc = memory.read_16(self.registers.pc + 1);

        self.last_executed_instruction = format!("CALL {:X}", self.registers.pc).to_string();

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 24;
    }

    /** 
     * If flag Z is reset, push address of next instruction onto stack and then jump to address nn.
     */
    pub fn call_nz_nn(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = format!("CALL NZ,{:X}", self.registers.pc).to_string();

        if self.registers.is_flag_z() {
            self.pc_to_increment = 3;
            self.last_instruction_ccycles = 12;
        }

        let next_pc :u16 = self.registers.pc + 3;
        self.push_vv(memory, next_pc);
        
        self.registers.pc = memory.read_16(self.registers.pc + 1);
        
        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 24;
    }

    /**
     * Pop two bytes from stack & jump to that address.
     */
    pub fn ret(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "RET".to_string();

        self.registers.pc = self.pop_vv(memory);

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Pop two bytes from stack & jump to that address, enabling interruptions.
     */
    pub fn reti(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "RETI".to_string();

        self.registers.pc = self.pop_vv(memory);

        self.ime = true;

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Pop two bytes from stack & jump to that address if flag Z is not set.
     */
    pub fn ret_nz(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "RET NZ".to_string();

        if !self.registers.is_flag_z() {
            self.registers.pc = self.pop_vv(memory);
            self.last_instruction_ccycles = 20;
        } else {
            self.registers.pc += 1;
            self.last_instruction_ccycles = 8;
        }
        
        self.pc_to_increment = 0;
    }

    /**
     * Pop two bytes from stack & jump to that address if flag Z is set.
     */
    pub fn ret_z(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "RET Z".to_string();

        if self.registers.is_flag_z() {
            self.registers.pc = self.pop_vv(memory);
            self.last_instruction_ccycles = 20;
        } else {
            self.registers.pc += 1;
            self.last_instruction_ccycles = 8;
        }

        self.pc_to_increment = 0;
    }

    /**
     * Pop two bytes from stack & jump to that address if flag C is not set.
     */
    pub fn ret_nc(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "RET NC".to_string();

        if !self.registers.is_flag_c() {
            self.registers.pc = self.pop_vv(memory);
            self.last_instruction_ccycles = 20;
        } else {
            self.registers.pc += 1;
            self.last_instruction_ccycles = 8;
        }
        
        self.pc_to_increment = 0;
    }

    /**
    * Pop two bytes from stack & jump to that address if flag C is set.
    */
    pub fn ret_c(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "RET C".to_string();

        if self.registers.is_flag_c() {
            self.registers.pc = self.pop_vv(memory);
            self.last_instruction_ccycles = 20;
        } else {
            self.registers.pc += 1;
            self.last_instruction_ccycles = 8;
        }

        self.pc_to_increment = 0;
    }

    // --- RESTART INSTRUCTIONS ------------------------------------------------------------------------------------------------------------

    pub fn rst_18(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "RST $18".to_string();
        self.rst_v(memory, 0x18)
    }

    pub fn rst_38(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "RST $38".to_string();
        self.rst_v(memory, 0x38);
    }

    fn rst_28(&mut self, memory : &mut Memory) {
        self.last_executed_instruction = "RST $28".to_string();
        self.rst_v(memory, 0x28);
    }


    // --- STACK INSTRUCTIONS --------------------------------------------------------------------------------------------------------------

    /**
     * Push HL into stack.
     */
    pub fn push_hl(&mut self, memory : &mut Memory) {
        self.last_executed_instruction = "PUSH HL".to_string();
        let reg: u16 = self.registers.read_hl();
        self.push_vv(memory, reg);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Push BC into stack.
     */
    pub fn push_bc(&mut self, memory : &mut Memory) {
        self.last_executed_instruction = "PUSH BC".to_string();
        let reg: u16 = self.registers.read_bc();
        self.push_vv(memory, reg);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Push AF into stack.
     */
    pub fn push_af(&mut self, memory : &mut Memory) {
        self.last_executed_instruction = "PUSH AF".to_string();
        let reg: u16 = self.registers.read_word(&WordRegister::AF);
        self.push_vv(memory, reg);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Push DE into stack.
     */
    pub fn push_de(&mut self, memory : &mut Memory) {
        self.last_executed_instruction = "PUSH DE".to_string();
        let reg: u16 = self.registers.read_de();
        self.push_vv(memory, reg);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Pops stack to AF.
     */
    pub fn pop_af(&mut self, memory : &mut Memory) {
        self.last_executed_instruction = "POP AF".to_string();

        let popped: u16 = self.pop_vv(memory);
        self.registers.write_word(&WordRegister::AF, popped);
        
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 12;
    }

    /**
     * Pops stack to AF.
     */
    pub fn pop_bc(&mut self, memory : &mut Memory) {
        self.last_executed_instruction = "POP BC".to_string();

        let popped: u16 = self.pop_vv(memory);
        self.registers.write_bc(popped);
        
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 12;
    }

    /**
     * Pops stack to DE.
     */
    pub fn pop_de(&mut self, memory : &mut Memory) {
        self.last_executed_instruction = "POP DE".to_string();

        let popped: u16 = self.pop_vv(memory);
        self.registers.write_de(popped);
        
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 12;
    }

    /**
     * Pops stack to HL.
     */
    pub fn pop_hl(&mut self, memory : &mut Memory) {
        self.last_executed_instruction = "POP HL".to_string();

        let popped: u16 = self.pop_vv(memory);
        self.registers.write_hl(popped);
        
        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 12;
    }

    // --- PREFIX CB -----------------------------------------------------------------------------------------------------------------------

    pub fn prefix_cb(&mut self, memory : &mut Memory)
    {
        let op: u8 = memory.read_8(self.registers.pc + 1);

        match op {
            0x11 => self.rl_c(),
            0x19 => self.rr_c(),
            0x1A => self.rr_d(),
            0x27 => self.sla_a(),
            0x37 => self.swap_a(),
            0x3F => self.srl_a(),
            0x40 => self.bit_v_r(0, ByteRegister::B),
            0x41 => self.bit_v_r(0, ByteRegister::C),
            0x42 => self.bit_v_r(0, ByteRegister::D),
            0x43 => self.bit_v_r(0, ByteRegister::E),
            0x44 => self.bit_v_r(0, ByteRegister::H),
            0x45 => self.bit_v_r(0, ByteRegister::L),
            0x46 => self.bit_v_mhl(memory, 0),
            0x47 => self.bit_v_r(0, ByteRegister::A),
            0x48 => self.bit_v_r(1, ByteRegister::B),
            0x49 => self.bit_v_r(1, ByteRegister::C),
            0x4A => self.bit_v_r(1, ByteRegister::D),
            0x4B => self.bit_v_r(1, ByteRegister::E),
            0x4C => self.bit_v_r(1, ByteRegister::H),
            0x4D => self.bit_v_r(1, ByteRegister::L),
            0x4E => self.bit_v_mhl(memory, 1),
            0x4F => self.bit_v_r(1, ByteRegister::A),
            0x50 => self.bit_v_r(2, ByteRegister::B),
            0x51 => self.bit_v_r(2, ByteRegister::C),
            0x52 => self.bit_v_r(2, ByteRegister::D),
            0x53 => self.bit_v_r(2, ByteRegister::E),
            0x54 => self.bit_v_r(2, ByteRegister::H),
            0x55 => self.bit_v_r(2, ByteRegister::L),
            0x56 => self.bit_v_mhl(memory, 2),
            0x57 => self.bit_v_r(2, ByteRegister::A),
            0x58 => self.bit_v_r(3, ByteRegister::B),
            0x59 => self.bit_v_r(3, ByteRegister::C),
            0x5A => self.bit_v_r(3, ByteRegister::D),
            0x5B => self.bit_v_r(3, ByteRegister::E),
            0x5C => self.bit_v_r(3, ByteRegister::H),
            0x5D => self.bit_v_r(3, ByteRegister::L),
            0x5E => self.bit_v_mhl(memory, 3),
            0x5F => self.bit_v_r(3, ByteRegister::A),
            0x60 => self.bit_v_r(4, ByteRegister::B),
            0x61 => self.bit_v_r(4, ByteRegister::C),
            0x62 => self.bit_v_r(4, ByteRegister::D),
            0x63 => self.bit_v_r(4, ByteRegister::E),
            0x64 => self.bit_v_r(4, ByteRegister::H),
            0x65 => self.bit_v_r(4, ByteRegister::L),
            0x66 => self.bit_v_mhl(memory, 4),
            0x67 => self.bit_v_r(4, ByteRegister::A),
            0x68 => self.bit_v_r(5, ByteRegister::B),
            0x69 => self.bit_v_r(5, ByteRegister::C),
            0x6A => self.bit_v_r(5, ByteRegister::D),
            0x6B => self.bit_v_r(5, ByteRegister::E),
            0x6C => self.bit_v_r(5, ByteRegister::H),
            0x6D => self.bit_v_r(5, ByteRegister::L),
            0x6E => self.bit_v_mhl(memory, 5),
            0x6F => self.bit_v_r(5, ByteRegister::A),
            0x70 => self.bit_v_r(6, ByteRegister::B),
            0x71 => self.bit_v_r(6, ByteRegister::C),
            0x72 => self.bit_v_r(6, ByteRegister::D),
            0x73 => self.bit_v_r(6, ByteRegister::E),
            0x74 => self.bit_v_r(6, ByteRegister::H),
            0x75 => self.bit_v_r(6, ByteRegister::L),
            0x76 => self.bit_v_mhl(memory, 6),
            0x77 => self.bit_v_r(6, ByteRegister::A),
            0x78 => self.bit_v_r(7, ByteRegister::B),
            0x79 => self.bit_v_r(7, ByteRegister::C),
            0x7A => self.bit_v_r(7, ByteRegister::D),
            0x7B => self.bit_v_r(7, ByteRegister::E),
            0x7C => self.bit_v_r(7, ByteRegister::H),
            0x7D => self.bit_v_r(7, ByteRegister::L),
            0x7E => self.bit_v_mhl(memory, 7),
            0x7F => self.bit_v_r(7, ByteRegister::A),
            0x86 => self.res_0_mhl(memory),
            0x87 => self.res_0_a(),
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
        self.last_executed_instruction = "RR C".to_string();
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
     * Rotate left through carry register C.
     */
    pub fn rl_c(&mut self)
    {
        self.last_executed_instruction = "RL C".to_string();
        let new_carry: bool = self.registers.c & 0b10000000 == 0b10000000;

        self.registers.c = (self.registers.c << 1) | (0b00000001 & (self.registers.is_flag_c() as u8));

        let zero :bool = self.registers.c == 0;
        self.registers.set_flag_z(zero);
        self.registers.set_flag_c(new_carry);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Rotate left through carry register A.
     */
    pub fn rla(&mut self)
    {
        self.last_executed_instruction = "RLA".to_string();
        let new_carry: bool = self.registers.a & 0b10000000 == 0b10000000;

        self.registers.a = self.registers.a << 1;
        self.registers.a = self.registers.a | (0b00000001 & (self.registers.is_flag_c() as u8));

        let zero :bool = self.registers.a == 0;
        self.registers.set_flag_z(zero);
        self.registers.set_flag_c(new_carry);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /** 
     * Rotate right through carry register D.
     */
    pub fn rr_d(&mut self)
    {
        self.last_executed_instruction = "RR D".to_string();
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
     * Shifts bit 0 of register A to Carry, resets other flags, Z gets updated and MSF is preserved.
     */
    pub fn srl_a(&mut self)
    {
        self.last_executed_instruction = "SRL A".to_string();
        let carry : bool = self.registers.a & 0b1 == 1;
        let msf : u8 = self.registers.a & 0b10000000;

        self.registers.a = msf | ((self.registers.a >> 1) & 0b01111111);

        let zero :bool = self.registers.a == 0;

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
        self.last_executed_instruction = "SRL B".to_string();
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

    /**
     * Shift n left into Carry. LSB of n set to 0.
     */
    pub fn sla_a(&mut self)
    {
        self.last_executed_instruction = "SLA A".to_string();

        let carry : bool = self.registers.a & 0b10000000 == 0b10000000;

        self.registers.a = self.registers.a << 1;

        let zero :bool = self.registers.a == 0;

        self.registers.set_flag_z(zero);
        self.registers.set_flag_c(carry);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn swap_r(&mut self, value: u8, register_letter:  &str ) {
        self.last_executed_instruction = "SWAP A".to_string();

        let old_value = self.registers.a;
        self.registers.a = self.alu.swap_n(&mut self.registers, old_value);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn swap_a(&mut self) {
        self.last_executed_instruction = "SWAP A".to_string();

        let old_value = self.registers.a;
        self.registers.a = self.alu.swap_n(&mut self.registers, old_value);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn res_0_a(&mut self) {
        self.last_executed_instruction = "RES 0,A".to_string();

        let value = self.registers.a & 0b11111110;
        self.registers.a = value;

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn res_0_mhl(&mut self, memory: &mut Memory) {
        self.last_executed_instruction = "RES 0,(HL)".to_string();

        let mut value = memory.read_8(self.registers.read_hl());
        value &= 0b11111110;
        memory.write_8(self.registers.read_hl(), value);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 16;
    }

    fn daa(&mut self) {
        self.last_executed_instruction = "DAA".to_string();

        if !self.registers.is_flag_n() {
            // Addition
            if self.registers.is_flag_c() || self.registers.a > 0x99 {
                self.registers.a += 0x60;
                self.registers.set_flag_c(true);
            }

            if self.registers.is_flag_h() || (self.registers.a & 0x0f) > 0x09 {
                self.registers.a += 0x6;
            }
        } else {
            if self.registers.is_flag_c() {
                self.registers.a -= 0x60;
            }

            if self.registers.is_flag_h() {
                self.registers.a -= 0x6;
            }
        }

        self.registers.set_flag_z(self.registers.a == 0);
        self.registers.set_flag_h(false);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }


    // --- INTERNAL --------------------------------------------------------------------------------

    fn push_vv(&mut self, memory : &mut Memory, value: u16) {
        memory.write_16(self.registers.sp - 2, value);
        self.registers.sp = self.registers.sp - 2;
    }

    fn pop_vv(&mut self, memory : &mut Memory) -> u16 {
        let value = memory.read_16(self.registers.sp);
        self.registers.sp += 2;

        return value;
    }

    fn rst_v(&mut self, memory: &mut Memory, value: u8) {
        self.push_vv(memory, self.registers.pc + 1);

        self.registers.pc = value as u16;

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 16;
    }

    fn interrupt_vv(&mut self, memory: &mut Memory, new_address: u16) {
        self.ime = false;
        self.push_vv(memory, self.registers.pc);
        self.registers.pc = new_address;
    }

    fn bit_v_r(&mut self, bit: u8, register: ByteRegister) {
        self.last_executed_instruction = format!("BIT {},{}", bit, register.to_string()).to_string();

        let mask = 1u8 << bit;
        let value = self.registers.read_byte(&register);

        let zero = value & mask != mask;

        self.registers.set_flag_z(zero);
        self.registers.set_flag_n(false);
        self.registers.set_flag_h(true);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn bit_v_mhl(&mut self, memory: &mut Memory, bit: u8) {
        self.last_executed_instruction = format!("BIT {},(HL)", bit).to_string();

        let mask = 1u8 << bit;
        let value = memory.read_8(self.registers.read_hl());

        let zero = value & mask != mask;

        self.registers.set_flag_z(zero);
        self.registers.set_flag_n(false);
        self.registers.set_flag_h(true);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 16;
    }


    // --- INTERRUPTS ----------------------------------------------------------------------------------

    pub fn are_interrupts_enabled(&self) -> bool {
        self.ime
    }

    pub fn vblank_interrupt(&mut self, memory: &mut Memory) {
        memory.interrupt_flag().set_vblank(false);

        self.interrupt_vv(memory, 0x40)
    }

    /**
     * Disables interrupts
     */
    fn di(&mut self) {
        self.last_executed_instruction = "DI".to_string();

        self.ime = false;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Enables interrupts
     */
    fn ei(&mut self) {
        self.last_executed_instruction = "EI".to_string();

        self.ime = true;

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    // --- HALT ------------------------------------------------------------------------------------

    pub fn is_halted(&self) -> bool {
        self.halted
    }

    pub fn unhalt(&mut self) {
        self.halted = false;
    }

    fn halt(&mut self) {
        if self.ime {
            self.halted = true;

            self.pc_to_increment = 1;
            self.last_instruction_ccycles = 4;
        } else {
            self.pc_to_increment = 2;
            self.last_instruction_ccycles = 4;
        }
    }
}