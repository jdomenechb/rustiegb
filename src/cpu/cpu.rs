use super::alu::ALU;
use super::registers::CPURegisters;
use crate::cpu::registers::{ByteRegister, WordRegister};
use crate::memory::memory::Memory;
use std::num::Wrapping;
use std::sync::{RwLock, Arc};

pub struct CPU {
    memory: Arc<RwLock<Memory>>,

    pub registers: CPURegisters,
    alu: ALU,
    trace: bool,
    available_cycles: i32,

    pc_to_increment: i8,
    last_instruction_ccycles: i16,
    debug: bool,
    last_executed_instruction: String,
    ime: bool,
    halted: bool,
}

impl CPU {
    const AVAILABLE_CCYCLES_PER_FRAME: i32 = 70221;

    pub fn new(memory: Arc<RwLock<Memory>>, debug: bool, bootstrap: bool) -> CPU {
        return CPU {
            memory,

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
        };
    }

    pub fn reset_available_ccycles(&mut self) {
        self.available_cycles = CPU::AVAILABLE_CCYCLES_PER_FRAME;
    }

    pub fn has_available_ccycles(&self) -> bool {
        return self.available_cycles > 0;
    }

    pub fn get_last_instruction_ccycles(&self) -> u8 {
        debug_assert!(
            self.last_instruction_ccycles >= 0,
            "No instruction has been executed yet"
        );

        return self.last_instruction_ccycles as u8;
    }

    pub fn step(&mut self) {
        self.pc_to_increment = -1;
        self.last_instruction_ccycles = -1;

        let instruction;

        {
            let mut memory = self.memory.write().unwrap();

            if self.registers.pc == 0x100 && memory.has_bootstrap_rom() {
                memory.erase_bootstrap_rom();
            }

            instruction = memory.read_8(self.registers.pc);
        }

        let current_pc = self.registers.pc;

        match instruction {
            0x00 => self.nop(),
            0x01 => self.ld_rr_nn(WordRegister::BC),
            0x02 => self.ld_mrr_r(WordRegister::BC, ByteRegister::A),
            0x03 => self.inc_rr(WordRegister::BC),
            0x04 => self.inc_r(ByteRegister::B),
            0x05 => self.dec_r(ByteRegister::B),
            0x06 => self.ld_r_n(ByteRegister::B),
            0x07 => self.rlca(),

            0x08 => self.ld_mnn_sp(),
            0x09 => self.add_hl_rr(WordRegister::BC),
            0x0A => self.ld_r_mrr(ByteRegister::A, WordRegister::BC),
            0x0B => self.dec_rr(WordRegister::BC),
            0x0C => self.inc_r(ByteRegister::C),
            0x0D => self.dec_r(ByteRegister::C),
            0x0E => self.ld_r_n(ByteRegister::C),

            0x11 => self.ld_rr_nn(WordRegister::DE),
            0x12 => self.ld_mrr_r( WordRegister::DE, ByteRegister::A),
            0x13 => self.inc_rr(WordRegister::DE),
            0x14 => self.inc_r(ByteRegister::D),
            0x15 => self.dec_r(ByteRegister::D),
            0x16 => self.ld_r_n(ByteRegister::D),
            0x17 => self.rla(),

            0x18 => self.jr_n(),
            0x19 => self.add_hl_rr(WordRegister::DE),
            0x1A => self.ld_r_mrr(ByteRegister::A, WordRegister::DE),
            0x1B => self.dec_rr(WordRegister::DE),
            0x1C => self.inc_r(ByteRegister::E),
            0x1D => self.dec_r(ByteRegister::E),
            0x1E => self.ld_r_n(ByteRegister::E),
            0x1F => self.rra(),

            0x20 => self.jr_nz_n(),
            0x21 => self.ld_rr_nn(WordRegister::HL),
            0x22 => self.ldi_mhl_a(),
            0x23 => self.inc_rr(WordRegister::HL),
            0x24 => self.inc_r(ByteRegister::H),
            0x25 => self.dec_r(ByteRegister::H),
            0x26 => self.ld_r_n(ByteRegister::H),
            0x27 => self.daa(),

            0x28 => self.jr_z_n(),
            0x29 => self.add_hl_rr(WordRegister::HL),
            0x2A => self.ldi_a_mhl(),
            0x2B => self.dec_rr(WordRegister::HL),
            0x2C => self.inc_r(ByteRegister::L),
            0x2D => self.dec_r(ByteRegister::L),
            0x2E => self.ld_r_n(ByteRegister::L),
            0x2F => self.cpl(),

            0x30 => self.jr_nc_n(),
            0x31 => self.ld_rr_nn(WordRegister::SP),
            0x32 => self.ldd_mhl_a(),
            0x33 => self.inc_rr(WordRegister::SP),
            0x34 => self.inc_mhl(),
            0x35 => self.dec_mhl(),
            0x36 => self.ld_mhl_n(),
            0x37 => self.scf(),

            0x38 => self.jr_c_n(),
            0x39 => self.add_hl_rr(WordRegister::SP),
            0x3A => self.ldd_a_mhl(),
            0x3B => self.dec_rr(WordRegister::SP),
            0x3C => self.inc_r(ByteRegister::A),
            0x3D => self.dec_r(ByteRegister::A),
            0x3E => self.ld_r_n(ByteRegister::A),
            0x3F => self.ccf(),

            0x40 => self.ld_r_r(ByteRegister::B, ByteRegister::B),
            0x41 => self.ld_r_r(ByteRegister::B, ByteRegister::C),
            0x42 => self.ld_r_r(ByteRegister::B, ByteRegister::D),
            0x43 => self.ld_r_r(ByteRegister::B, ByteRegister::E),
            0x44 => self.ld_r_r(ByteRegister::B, ByteRegister::H),
            0x45 => self.ld_r_r(ByteRegister::B, ByteRegister::L),
            0x46 => self.ld_r_mrr(ByteRegister::B, WordRegister::HL),
            0x47 => self.ld_r_r(ByteRegister::B, ByteRegister::A),

            0x48 => self.ld_r_r(ByteRegister::C, ByteRegister::B),
            0x49 => self.ld_r_r(ByteRegister::C, ByteRegister::C),
            0x4A => self.ld_r_r(ByteRegister::C, ByteRegister::D),
            0x4B => self.ld_r_r(ByteRegister::C, ByteRegister::E),
            0x4C => self.ld_r_r(ByteRegister::C, ByteRegister::H),
            0x4D => self.ld_r_r(ByteRegister::C, ByteRegister::L),
            0x4E => self.ld_r_mrr(ByteRegister::C, WordRegister::HL),
            0x4F => self.ld_r_r(ByteRegister::C, ByteRegister::A),

            0x50 => self.ld_r_r(ByteRegister::D, ByteRegister::B),
            0x51 => self.ld_r_r(ByteRegister::D, ByteRegister::C),
            0x52 => self.ld_r_r(ByteRegister::D, ByteRegister::D),
            0x53 => self.ld_r_r(ByteRegister::D, ByteRegister::E),
            0x54 => self.ld_r_r(ByteRegister::D, ByteRegister::H),
            0x55 => self.ld_r_r(ByteRegister::D, ByteRegister::L),
            0x56 => self.ld_r_mrr(ByteRegister::D, WordRegister::HL),
            0x57 => self.ld_r_r(ByteRegister::D, ByteRegister::A),

            0x58 => self.ld_r_r(ByteRegister::E, ByteRegister::B),
            0x59 => self.ld_r_r(ByteRegister::E, ByteRegister::C),
            0x5A => self.ld_r_r(ByteRegister::E, ByteRegister::D),
            0x5B => self.ld_r_r(ByteRegister::E, ByteRegister::E),
            0x5C => self.ld_r_r(ByteRegister::E, ByteRegister::H),
            0x5D => self.ld_r_r(ByteRegister::E, ByteRegister::L),
            0x5E => self.ld_r_mrr(ByteRegister::E, WordRegister::HL),
            0x5F => self.ld_r_r(ByteRegister::E, ByteRegister::A),

            0x60 => self.ld_r_r(ByteRegister::H, ByteRegister::B),
            0x61 => self.ld_r_r(ByteRegister::H, ByteRegister::C),
            0x62 => self.ld_r_r(ByteRegister::H, ByteRegister::D),
            0x63 => self.ld_r_r(ByteRegister::H, ByteRegister::E),
            0x64 => self.ld_r_r(ByteRegister::H, ByteRegister::H),
            0x65 => self.ld_r_r(ByteRegister::H, ByteRegister::L),
            0x66 => self.ld_r_mrr(ByteRegister::H, WordRegister::HL),
            0x67 => self.ld_r_r(ByteRegister::H, ByteRegister::A),

            0x68 => self.ld_r_r(ByteRegister::L, ByteRegister::B),
            0x69 => self.ld_r_r(ByteRegister::L, ByteRegister::C),
            0x6A => self.ld_r_r(ByteRegister::L, ByteRegister::D),
            0x6B => self.ld_r_r(ByteRegister::L, ByteRegister::E),
            0x6C => self.ld_r_r(ByteRegister::L, ByteRegister::H),
            0x6D => self.ld_r_r(ByteRegister::L, ByteRegister::L),
            0x6E => self.ld_r_mrr(ByteRegister::L, WordRegister::HL),
            0x6F => self.ld_r_r(ByteRegister::L, ByteRegister::A),

            0x70 => self.ld_mrr_r(WordRegister::HL, ByteRegister::B),
            0x71 => self.ld_mrr_r(WordRegister::HL, ByteRegister::C),
            0x72 => self.ld_mrr_r(WordRegister::HL, ByteRegister::D),
            0x73 => self.ld_mrr_r(WordRegister::HL, ByteRegister::E),
            0x74 => self.ld_mrr_r(WordRegister::HL, ByteRegister::H),
            0x75 => self.ld_mrr_r(WordRegister::HL, ByteRegister::L),
            0x76 => self.halt(),
            0x77 => self.ld_mrr_r(WordRegister::HL, ByteRegister::A),

            0x78 => self.ld_r_r(ByteRegister::A, ByteRegister::B),
            0x79 => self.ld_r_r(ByteRegister::A, ByteRegister::C),
            0x7A => self.ld_r_r(ByteRegister::A, ByteRegister::D),
            0x7B => self.ld_r_r(ByteRegister::A, ByteRegister::E),
            0x7C => self.ld_r_r(ByteRegister::A, ByteRegister::H),
            0x7D => self.ld_r_r(ByteRegister::A, ByteRegister::L),
            0x7E => self.ld_r_mrr(ByteRegister::A, WordRegister::HL),
            0x7F => self.ld_r_r(ByteRegister::A, ByteRegister::A),

            0x80 => self.add_a_r(ByteRegister::B),
            0x81 => self.add_a_r(ByteRegister::C),
            0x82 => self.add_a_r(ByteRegister::D),
            0x83 => self.add_a_r(ByteRegister::E),
            0x84 => self.add_a_r(ByteRegister::H),
            0x85 => self.add_a_r(ByteRegister::L),
            0x86 => self.add_a_mhl(),
            0x87 => self.add_a_r(ByteRegister::A),

            0x88 => self.adc_a_r(ByteRegister::B),
            0x89 => self.adc_a_r(ByteRegister::C),
            0x8A => self.adc_a_r(ByteRegister::D),
            0x8B => self.adc_a_r(ByteRegister::E),
            0x8C => self.adc_a_r(ByteRegister::H),
            0x8D => self.adc_a_r(ByteRegister::L),
            0x8E => self.adc_a_mhl(),
            0x8F => self.adc_a_r(ByteRegister::A),

            0x90 => self.sub_r(ByteRegister::B),
            0x91 => self.sub_r(ByteRegister::C),
            0x92 => self.sub_r(ByteRegister::D),
            0x93 => self.sub_r(ByteRegister::E),
            0x94 => self.sub_r(ByteRegister::H),
            0x95 => self.sub_r(ByteRegister::L),
            0x96 => self.sub_mhl(),
            0x97 => self.sub_r(ByteRegister::A),

            0x98 => self.sbc_r(ByteRegister::B),
            0x99 => self.sbc_r(ByteRegister::C),
            0x9A => self.sbc_r(ByteRegister::D),
            0x9B => self.sbc_r(ByteRegister::E),
            0x9C => self.sbc_r(ByteRegister::H),
            0x9D => self.sbc_r(ByteRegister::L),
            0x9E => self.sbc_mhl(),
            0x9F => self.sbc_r(ByteRegister::A),

            0xA0 => self.and_r(ByteRegister::B),
            0xA1 => self.and_r(ByteRegister::C),
            0xA2 => self.and_r(ByteRegister::D),
            0xA3 => self.and_r(ByteRegister::E),
            0xA4 => self.and_r(ByteRegister::H),
            0xA5 => self.and_r(ByteRegister::L),
            0xA6 => self.and_mhl(),
            0xA7 => self.and_r(ByteRegister::A),

            0xA8 => self.xor_r(ByteRegister::B),
            0xA9 => self.xor_r(ByteRegister::C),
            0xAA => self.xor_r(ByteRegister::D),
            0xAB => self.xor_r(ByteRegister::E),
            0xAC => self.xor_r(ByteRegister::H),
            0xAD => self.xor_r(ByteRegister::L),
            0xAE => self.xor_mhl(),
            0xAF => self.xor_r(ByteRegister::A),

            0xB0 => self.or_r(ByteRegister::B),
            0xB1 => self.or_r(ByteRegister::C),
            0xB2 => self.or_r(ByteRegister::D),
            0xB3 => self.or_r(ByteRegister::E),
            0xB4 => self.or_r(ByteRegister::H),
            0xB5 => self.or_r(ByteRegister::L),
            0xB6 => self.or_mhl(),
            0xB7 => self.or_r(ByteRegister::A),

            0xB8 => self.cp_r(ByteRegister::B),
            0xB9 => self.cp_r(ByteRegister::C),
            0xBA => self.cp_r(ByteRegister::D),
            0xBB => self.cp_r(ByteRegister::E),
            0xBC => self.cp_r(ByteRegister::H),
            0xBD => self.cp_r(ByteRegister::L),
            0xBE => self.cp_mhl(),
            0xBF => self.cp_r(ByteRegister::A),

            0xC0 => self.ret_nz(),
            0xC1 => self.pop_rr(WordRegister::BC),
            0xC2 => self.jp_nz_nn(),
            0xC3 => self.jp_nn(),
            0xC4 => self.call_nz_nn(),
            0xC5 => self.push_rr(WordRegister::BC),
            0xC6 => self.add_a_n(),
            0xC7 => self.rst_v_w_out(0),

            0xC8 => self.ret_z(),
            0xC9 => self.ret(),
            0xCA => self.jp_z_nn(),
            0xCB => self.prefix_cb(),
            0xCC => self.call_z_nn(),
            0xCD => self.call(),
            0xCE => self.adc_a_n(),
            0xCF => self.rst_v_w_out(0x08),

            0xD0 => self.ret_nc(),
            0xD1 => self.pop_rr(WordRegister::DE),
            0xD2 => self.jp_nc_nn(),
            0xD4 => self.call_nc_nn(),
            0xD5 => self.push_rr(WordRegister::DE),
            0xD6 => self.sub_n(),
            0xD7 => self.rst_v_w_out(0x10),

            0xD8 => self.ret_c(),
            0xD9 => self.reti(),
            0xDA => self.jp_c_nn(),
            0xDC => self.call_c_nn(),
            0xDE => self.sbc_a_n(),
            0xDF => self.rst_v_w_out(0x18),

            0xE0 => self.ldh_n_a(),
            0xE1 => self.pop_rr(WordRegister::HL),
            0xE2 => self.ld_mc_a(),
            0xE5 => self.push_rr(WordRegister::HL),
            0xE6 => self.and_n(),
            0xE7 => self.rst_v_w_out(0x20),

            0xE8 => self.add_sp_n(),
            0xE9 => self.jp_mhl(),
            0xEA => self.ld_nn_a(),
            0xEE => self.xor_n(),
            0xEF => self.rst_v_w_out(0x28),

            0xF0 => self.ldh_a_n(),
            0xF1 => self.pop_rr(WordRegister::AF),
            0xF2 => self.ld_a_mc(),
            0xF3 => self.di(),
            0xF5 => self.push_rr(WordRegister::AF),
            0xF6 => self.or_n(),
            0xF7 => self.rst_v_w_out(0x30),

            0xF8 => self.ld_hl_sp_n(),
            0xF9 => self.ld_sp_hl(),
            0xFA => self.ld_a_nn(),
            0xFB => self.ei(),
            0xFE => self.cp_n(),
            0xFF => self.rst_v_w_out(0x38),
            _ => {
                panic!("Instruction not implemented: {:X}", instruction);
            }
        }

        debug_assert!(
            self.last_instruction_ccycles >= 0,
            "Instruction does not count ccycles: {:X}",
            instruction
        );
        debug_assert!(
            self.pc_to_increment >= 0,
            "Instruction does not increment PC: {:X}",
            instruction
        );

        if self.debug {
            println!("{:X}: {}", current_pc, self.last_executed_instruction);
        }

        self.available_cycles -= self.last_instruction_ccycles as i32;

        self.registers.pc += self.pc_to_increment as u16;

        // TODO: enable when ready
        //memory.step(self.last_instruction_ccycles);
    }

    fn prefix_cb(&mut self) {
        let op: u8;

        {
            let memory = self.memory.read().unwrap();
            op = memory.read_8(self.registers.pc + 1);
        }

        match op {
            0x00 => self.rlc_r(ByteRegister::B),
            0x01 => self.rlc_r(ByteRegister::C),
            0x02 => self.rlc_r(ByteRegister::D),
            0x03 => self.rlc_r(ByteRegister::E),
            0x04 => self.rlc_r(ByteRegister::H),
            0x05 => self.rlc_r(ByteRegister::L),
            0x06 => self.rlc_mrr(WordRegister::HL),
            0x07 => self.rlc_r(ByteRegister::A),

            0x10 => self.rl_r(ByteRegister::B),
            0x11 => self.rl_r(ByteRegister::C),
            0x12 => self.rl_r(ByteRegister::D),
            0x13 => self.rl_r(ByteRegister::E),
            0x14 => self.rl_r(ByteRegister::H),
            0x15 => self.rl_r(ByteRegister::L),
            0x17 => self.rl_r(ByteRegister::A),

            0x18 => self.rr_r(ByteRegister::B),
            0x19 => self.rr_r(ByteRegister::C),
            0x1A => self.rr_r(ByteRegister::D),
            0x1B => self.rr_r(ByteRegister::E),
            0x1C => self.rr_r(ByteRegister::H),
            0x1D => self.rr_r(ByteRegister::L),
            0x1F => self.rr_r(ByteRegister::A),

            0x20 => self.sla_r(ByteRegister::B),
            0x21 => self.sla_r(ByteRegister::C),
            0x22 => self.sla_r(ByteRegister::D),
            0x23 => self.sla_r(ByteRegister::E),
            0x24 => self.sla_r(ByteRegister::H),
            0x25 => self.sla_r(ByteRegister::L),
            0x27 => self.sla_r(ByteRegister::A),

            0x30 => self.swap_r(ByteRegister::B),
            0x31 => self.swap_r(ByteRegister::C),
            0x32 => self.swap_r(ByteRegister::D),
            0x33 => self.swap_r(ByteRegister::E),
            0x34 => self.swap_r(ByteRegister::H),
            0x35 => self.swap_r(ByteRegister::L),
            0x37 => self.swap_r(ByteRegister::A),

            0x38 => self.srl_r(ByteRegister::B),
            0x39 => self.srl_r(ByteRegister::C),
            0x3A => self.srl_r(ByteRegister::D),
            0x3B => self.srl_r(ByteRegister::E),
            0x3C => self.srl_r(ByteRegister::H),
            0x3D => self.srl_r(ByteRegister::L),

            0x3F => self.srl_r(ByteRegister::A),

            0x40 => self.bit_v_r(0, ByteRegister::B),
            0x41 => self.bit_v_r(0, ByteRegister::C),
            0x42 => self.bit_v_r(0, ByteRegister::D),
            0x43 => self.bit_v_r(0, ByteRegister::E),
            0x44 => self.bit_v_r(0, ByteRegister::H),
            0x45 => self.bit_v_r(0, ByteRegister::L),
            0x46 => self.bit_v_mhl(0),
            0x47 => self.bit_v_r(0, ByteRegister::A),

            0x48 => self.bit_v_r(1, ByteRegister::B),
            0x49 => self.bit_v_r(1, ByteRegister::C),
            0x4A => self.bit_v_r(1, ByteRegister::D),
            0x4B => self.bit_v_r(1, ByteRegister::E),
            0x4C => self.bit_v_r(1, ByteRegister::H),
            0x4D => self.bit_v_r(1, ByteRegister::L),
            0x4E => self.bit_v_mhl(1),
            0x4F => self.bit_v_r(1, ByteRegister::A),

            0x50 => self.bit_v_r(2, ByteRegister::B),
            0x51 => self.bit_v_r(2, ByteRegister::C),
            0x52 => self.bit_v_r(2, ByteRegister::D),
            0x53 => self.bit_v_r(2, ByteRegister::E),
            0x54 => self.bit_v_r(2, ByteRegister::H),
            0x55 => self.bit_v_r(2, ByteRegister::L),
            0x56 => self.bit_v_mhl(2),
            0x57 => self.bit_v_r(2, ByteRegister::A),

            0x58 => self.bit_v_r(3, ByteRegister::B),
            0x59 => self.bit_v_r(3, ByteRegister::C),
            0x5A => self.bit_v_r(3, ByteRegister::D),
            0x5B => self.bit_v_r(3, ByteRegister::E),
            0x5C => self.bit_v_r(3, ByteRegister::H),
            0x5D => self.bit_v_r(3, ByteRegister::L),
            0x5E => self.bit_v_mhl(3),
            0x5F => self.bit_v_r(3, ByteRegister::A),

            0x60 => self.bit_v_r(4, ByteRegister::B),
            0x61 => self.bit_v_r(4, ByteRegister::C),
            0x62 => self.bit_v_r(4, ByteRegister::D),
            0x63 => self.bit_v_r(4, ByteRegister::E),
            0x64 => self.bit_v_r(4, ByteRegister::H),
            0x65 => self.bit_v_r(4, ByteRegister::L),
            0x66 => self.bit_v_mhl(4),
            0x67 => self.bit_v_r(4, ByteRegister::A),

            0x68 => self.bit_v_r(5, ByteRegister::B),
            0x69 => self.bit_v_r(5, ByteRegister::C),
            0x6A => self.bit_v_r(5, ByteRegister::D),
            0x6B => self.bit_v_r(5, ByteRegister::E),
            0x6C => self.bit_v_r(5, ByteRegister::H),
            0x6D => self.bit_v_r(5, ByteRegister::L),
            0x6E => self.bit_v_mhl(5),
            0x6F => self.bit_v_r(5, ByteRegister::A),

            0x70 => self.bit_v_r(6, ByteRegister::B),
            0x71 => self.bit_v_r(6, ByteRegister::C),
            0x72 => self.bit_v_r(6, ByteRegister::D),
            0x73 => self.bit_v_r(6, ByteRegister::E),
            0x74 => self.bit_v_r(6, ByteRegister::H),
            0x75 => self.bit_v_r(6, ByteRegister::L),
            0x76 => self.bit_v_mhl(6),
            0x77 => self.bit_v_r(6, ByteRegister::A),

            0x78 => self.bit_v_r(7, ByteRegister::B),
            0x79 => self.bit_v_r(7, ByteRegister::C),
            0x7A => self.bit_v_r(7, ByteRegister::D),
            0x7B => self.bit_v_r(7, ByteRegister::E),
            0x7C => self.bit_v_r(7, ByteRegister::H),
            0x7D => self.bit_v_r(7, ByteRegister::L),
            0x7E => self.bit_v_mhl(7),
            0x7F => self.bit_v_r(7, ByteRegister::A),

            0x80 => self.res_v_r(0, ByteRegister::B),
            0x81 => self.res_v_r(0, ByteRegister::C),
            0x82 => self.res_v_r(0, ByteRegister::D),
            0x83 => self.res_v_r(0, ByteRegister::E),
            0x84 => self.res_v_r(0, ByteRegister::H),
            0x85 => self.res_v_r(0, ByteRegister::L),
            0x86 => self.res_v_mhl(0),
            0x87 => self.res_v_r(0, ByteRegister::A),

            0x88 => self.res_v_r(1, ByteRegister::B),
            0x89 => self.res_v_r(1, ByteRegister::C),
            0x8A => self.res_v_r(1, ByteRegister::D),
            0x8B => self.res_v_r(1, ByteRegister::E),
            0x8C => self.res_v_r(1, ByteRegister::H),
            0x8D => self.res_v_r(1, ByteRegister::L),
            0x8E => self.res_v_mhl(1),
            0x8F => self.res_v_r(1, ByteRegister::A),

            0x90 => self.res_v_r(2, ByteRegister::B),
            0x91 => self.res_v_r(2, ByteRegister::C),
            0x92 => self.res_v_r(2, ByteRegister::D),
            0x93 => self.res_v_r(2, ByteRegister::E),
            0x94 => self.res_v_r(2, ByteRegister::H),
            0x95 => self.res_v_r(2, ByteRegister::L),
            0x96 => self.res_v_mhl(2),
            0x97 => self.res_v_r(2, ByteRegister::A),

            0x98 => self.res_v_r(3, ByteRegister::B),
            0x99 => self.res_v_r(3, ByteRegister::C),
            0x9A => self.res_v_r(3, ByteRegister::D),
            0x9B => self.res_v_r(3, ByteRegister::E),
            0x9C => self.res_v_r(3, ByteRegister::H),
            0x9D => self.res_v_r(3, ByteRegister::L),
            0x9E => self.res_v_mhl(3),
            0x9F => self.res_v_r(3, ByteRegister::A),

            0xA0 => self.res_v_r(4, ByteRegister::B),
            0xA1 => self.res_v_r(4, ByteRegister::C),
            0xA2 => self.res_v_r(4, ByteRegister::D),
            0xA3 => self.res_v_r(4, ByteRegister::E),
            0xA4 => self.res_v_r(4, ByteRegister::H),
            0xA5 => self.res_v_r(4, ByteRegister::L),
            0xA6 => self.res_v_mhl(4),
            0xA7 => self.res_v_r(4, ByteRegister::A),

            0xA8 => self.res_v_r(5, ByteRegister::B),
            0xA9 => self.res_v_r(5, ByteRegister::C),
            0xAA => self.res_v_r(5, ByteRegister::D),
            0xAB => self.res_v_r(5, ByteRegister::E),
            0xAC => self.res_v_r(5, ByteRegister::H),
            0xAD => self.res_v_r(5, ByteRegister::L),
            0xAE => self.res_v_mhl(5),
            0xAF => self.res_v_r(5, ByteRegister::A),

            0xB0 => self.res_v_r(6, ByteRegister::B),
            0xB1 => self.res_v_r(6, ByteRegister::C),
            0xB2 => self.res_v_r(6, ByteRegister::D),
            0xB3 => self.res_v_r(6, ByteRegister::E),
            0xB4 => self.res_v_r(6, ByteRegister::H),
            0xB5 => self.res_v_r(6, ByteRegister::L),
            0xB6 => self.res_v_mhl(6),
            0xB7 => self.res_v_r(6, ByteRegister::A),

            0xB8 => self.res_v_r(7, ByteRegister::B),
            0xB9 => self.res_v_r(7, ByteRegister::C),
            0xBA => self.res_v_r(7, ByteRegister::D),
            0xBB => self.res_v_r(7, ByteRegister::E),
            0xBC => self.res_v_r(7, ByteRegister::H),
            0xBD => self.res_v_r(7, ByteRegister::L),
            0xBE => self.res_v_mhl(7),
            0xBF => self.res_v_r(7, ByteRegister::A),

            0xC0 => self.set_v_r(0, ByteRegister::B),
            0xC1 => self.set_v_r(0, ByteRegister::C),
            0xC2 => self.set_v_r(0, ByteRegister::D),
            0xC3 => self.set_v_r(0, ByteRegister::E),
            0xC4 => self.set_v_r(0, ByteRegister::H),
            0xC5 => self.set_v_r(0, ByteRegister::L),
            0xC6 => self.set_v_mhl(0),
            0xC7 => self.set_v_r(0, ByteRegister::A),

            0xC8 => self.set_v_r(1, ByteRegister::B),
            0xC9 => self.set_v_r(1, ByteRegister::C),
            0xCA => self.set_v_r(1, ByteRegister::D),
            0xCB => self.set_v_r(1, ByteRegister::E),
            0xCC => self.set_v_r(1, ByteRegister::H),
            0xCD => self.set_v_r(1, ByteRegister::L),
            0xCE => self.set_v_mhl(1),
            0xCF => self.set_v_r(1, ByteRegister::A),

            0xD0 => self.set_v_r(2, ByteRegister::B),
            0xD1 => self.set_v_r(2, ByteRegister::C),
            0xD2 => self.set_v_r(2, ByteRegister::D),
            0xD3 => self.set_v_r(2, ByteRegister::E),
            0xD4 => self.set_v_r(2, ByteRegister::H),
            0xD5 => self.set_v_r(2, ByteRegister::L),
            0xD6 => self.set_v_mhl(2),
            0xD7 => self.set_v_r(2, ByteRegister::A),

            0xD8 => self.set_v_r(3, ByteRegister::B),
            0xD9 => self.set_v_r(3, ByteRegister::C),
            0xDA => self.set_v_r(3, ByteRegister::D),
            0xDB => self.set_v_r(3, ByteRegister::E),
            0xDC => self.set_v_r(3, ByteRegister::H),
            0xDD => self.set_v_r(3, ByteRegister::L),
            0xDE => self.set_v_mhl(3),
            0xDF => self.set_v_r(3, ByteRegister::A),

            0xE0 => self.set_v_r(4, ByteRegister::B),
            0xE1 => self.set_v_r(4, ByteRegister::C),
            0xE2 => self.set_v_r(4, ByteRegister::D),
            0xE3 => self.set_v_r(4, ByteRegister::E),
            0xE4 => self.set_v_r(4, ByteRegister::H),
            0xE5 => self.set_v_r(4, ByteRegister::L),
            0xE6 => self.set_v_mhl(4),
            0xE7 => self.set_v_r(4, ByteRegister::A),

            0xE8 => self.set_v_r(5, ByteRegister::B),
            0xE9 => self.set_v_r(5, ByteRegister::C),
            0xEA => self.set_v_r(5, ByteRegister::D),
            0xEB => self.set_v_r(5, ByteRegister::E),
            0xEC => self.set_v_r(5, ByteRegister::H),
            0xED => self.set_v_r(5, ByteRegister::L),
            0xEE => self.set_v_mhl(5),
            0xEF => self.set_v_r(5, ByteRegister::A),

            0xF0 => self.set_v_r(6, ByteRegister::B),
            0xF1 => self.set_v_r(6, ByteRegister::C),
            0xF2 => self.set_v_r(6, ByteRegister::D),
            0xF3 => self.set_v_r(6, ByteRegister::E),
            0xF4 => self.set_v_r(6, ByteRegister::H),
            0xF5 => self.set_v_r(6, ByteRegister::L),
            0xF6 => self.set_v_mhl(6),
            0xF7 => self.set_v_r(6, ByteRegister::A),

            0xF8 => self.set_v_r(7, ByteRegister::B),
            0xF9 => self.set_v_r(7, ByteRegister::C),
            0xFA => self.set_v_r(7, ByteRegister::D),
            0xFB => self.set_v_r(7, ByteRegister::E),
            0xFC => self.set_v_r(7, ByteRegister::H),
            0xFD => self.set_v_r(7, ByteRegister::L),
            0xFE => self.set_v_mhl(7),
            0xFF => self.set_v_r(7, ByteRegister::A),

            _ => {
                panic!("CB Instruction not implemented: {:X}", op);
            }
        }
    }

    // --- INSTRUCTIONS ---------------------------------------------------------------------------------------------------------------------

    fn nop(&mut self) {
        self.last_executed_instruction = "NOP".to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    // --- ARITHMETIC INSTRUCTIONS ----------------------------------------------------------------------------------------------------------

    fn dec_r(&mut self, register: ByteRegister) {
        self.last_executed_instruction = format!("DEC {}", register.to_string()).to_string();

        let value = self.registers.read_byte(&register);
        let value = self.alu.dec_n(&mut self.registers, value);
        self.registers.write_byte(&register, value);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn dec_mhl(&mut self) {
        self.last_executed_instruction = "DEC (HL)".to_string();

        let mut memory = self.memory.write().unwrap();

        let value = memory.read_8(self.registers.read_word(&WordRegister::HL));
        let value = self.alu.dec_n(&mut self.registers, value);
        memory.write_8(self.registers.read_word(&WordRegister::HL), value);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 12;
    }

    fn dec_rr(&mut self, register: WordRegister) {
        self.last_executed_instruction =
            format!("DEC {}", register.to_string().to_uppercase()).to_string();

        let value = self.registers.read_word(&register);
        self.registers.write_word(&register, self.alu.dec_nn(value));

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn inc_rr(&mut self, register: WordRegister) {
        self.last_executed_instruction =
            format!("INC {}", register.to_string().to_uppercase()).to_string();

        let value = self.registers.read_word(&register);
        self.registers.write_word(&register, self.alu.inc_nn(value));

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn inc_r(&mut self, register: ByteRegister) {
        self.last_executed_instruction = format!("INC {}", register.to_string()).to_string();

        let value: u8 = self.registers.read_byte(&register);
        let value: u8 = self.alu.inc_n(&mut self.registers, value);
        self.registers.write_byte(&register, value);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn inc_mhl(&mut self) {
        self.last_executed_instruction = "INC (HL)".to_string();

        let mut memory = self.memory.write().unwrap();

        let position = self.registers.read_word(&WordRegister::HL);
        let value: u8 = memory.read_8(position);
        let value: u8 = self.alu.inc_n(&mut self.registers, value);
        memory.write_8(position, value);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 12;
    }

    fn adc_a_r(&mut self, register: ByteRegister) {
        let value1: u8 = self.registers.read_byte(&ByteRegister::A);
        let value2: u8 = self.registers.read_byte(&register);

        let had_carry = self.registers.is_flag_c();

        self.last_executed_instruction = format!("ADC A,{}", register.to_string()).to_string();

        let mut result: u8 = self.alu.add_n(&mut self.registers, value1, value2);

        if had_carry {
            result = self.alu.add_n(&mut self.registers, result, 1);
        }

        self.registers.write_byte(&ByteRegister::A, result);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn adc_a_mhl(&mut self) {
        let memory = self.memory.read().unwrap();

        let value1: u8 = self.registers.read_byte(&ByteRegister::A);
        let mut value2: u8 = memory.read_8(self.registers.read_word(&WordRegister::HL));

        self.last_executed_instruction = "ADC A,(HL)".to_string();

        value2 = value2.overflowing_add(self.registers.is_flag_c() as u8).0;

        let result: u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.write_byte(&ByteRegister::A, result);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn adc_a_n(&mut self) {
        let memory = self.memory.read().unwrap();

        let value1: u8 = self.registers.a;
        let mut value2: u8 = memory.read_8(self.registers.pc + 1);
        self.last_executed_instruction = format!("ADC A,{:X}", value2).to_string();

        value2 = value2.overflowing_add(self.registers.is_flag_c() as u8).0;

        let result: u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.a = result;

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn add_a_r(&mut self, register_v: ByteRegister) {
        let value1: u8 = self.registers.read_byte(&ByteRegister::A);
        let value2: u8 = self.registers.read_byte(&register_v);

        self.last_executed_instruction = format!("ADD A,{}", register_v.to_string()).to_string();

        let result: u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.write_byte(&ByteRegister::A, result);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn add_a_n(&mut self) {
        let memory = self.memory.read().unwrap();
        let value1: u8 = memory.read_8(self.registers.pc + 1);
        let value2: u8 = self.registers.read_byte(&ByteRegister::A);

        self.last_executed_instruction = format!("ADD A,{:X}", value1).to_string();

        let result: u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.write_byte(&ByteRegister::A, result);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn add_a_mhl(&mut self) {
        let memory = self.memory.read().unwrap();
        let value1: u8 = memory.read_8(self.registers.read_word(&WordRegister::HL));
        let value2: u8 = self.registers.read_byte(&ByteRegister::A);

        self.last_executed_instruction = "ADD A,(HL)".to_string();

        let result: u8 = self.alu.add_n(&mut self.registers, value1, value2);
        self.registers.write_byte(&ByteRegister::A, result);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn add_hl_rr(&mut self, register: WordRegister) {
        let value1 = self.registers.read_word(&WordRegister::HL);
        let value2 = self.registers.read_word(&register);

        self.last_executed_instruction = format!("ADD HL,{}", register.to_string()).to_string();

        let result = self.alu.add_nn(&mut self.registers, value1, value2);
        self.registers.write_word(&WordRegister::HL, result);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn add_sp_n(&mut self) {
        let memory = self.memory.read().unwrap();

        let value1 = self.registers.read_word(&WordRegister::SP);
        let value2 = memory.read_8_signed(self.registers.read_word(&WordRegister::PC) + 1);

        self.last_executed_instruction = format!("ADD SP,{}", value2).to_string();

        let result = self
            .alu
            .add_nn_signed(&mut self.registers, value1, value2 as i16);
        self.registers.write_word(&WordRegister::SP, result);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 16;
    }

    fn sub_n(&mut self) {
        let memory = self.memory.read().unwrap();

        let value = self.registers.a;
        let to_subtract: u8 = memory.read_8(self.registers.pc + 1);

        self.last_executed_instruction = format!("SUB A, {:X}", to_subtract).to_string();

        let value = self.alu.sub_n(&mut self.registers, value, to_subtract);
        self.registers.a = value;

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn sub_r(&mut self, register: ByteRegister) {
        self.last_executed_instruction = format!("SUB {}", register.to_string()).to_string();

        let value = self.registers.read_byte(&ByteRegister::A);
        let to_subtract: u8 = self.registers.read_byte(&register);
        let value = self.alu.sub_n(&mut self.registers, value, to_subtract);
        self.registers.write_byte(&ByteRegister::A, value);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn sub_mhl(&mut self) {
        self.last_executed_instruction = "SUB (HL)".to_string();

        let value = self.registers.read_byte(&ByteRegister::A);

        let memory = self.memory.read().unwrap();
        let to_subtract: u8 = memory.read_8(self.registers.read_word(&WordRegister::HL));

        let value = self.alu.sub_n(&mut self.registers, value, to_subtract);
        self.registers.write_byte(&ByteRegister::A, value);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn sbc_r(&mut self, register: ByteRegister) {
        self.last_executed_instruction = format!("SBC A,{}", register.to_string()).to_string();

        let value = self.registers.read_byte(&ByteRegister::A);
        let to_subtract: u8 = self.registers.read_byte(&register).overflowing_add(1).0;
        let value = self.alu.sub_n(&mut self.registers, value, to_subtract);
        self.registers.write_byte(&ByteRegister::A, value);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn sbc_mhl(&mut self) {
        self.last_executed_instruction = "SBC A,(HL)".to_string();

        let value = self.registers.read_byte(&ByteRegister::A);

        let memory = self.memory.read().unwrap();
        let to_subtract: u8 = memory
            .read_8(self.registers.read_word(&WordRegister::HL))
            .overflowing_add(1)
            .0;

        let value = self.alu.sub_n(&mut self.registers, value, to_subtract);
        self.registers.write_byte(&ByteRegister::A, value);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn sbc_a_n(&mut self) {
        let memory = self.memory.read().unwrap();

        let value1 = self.registers.read_byte(&ByteRegister::A);
        let mut value2 = memory.read_8(self.registers.read_word(&WordRegister::PC) + 1);

        self.last_executed_instruction = format!("SBC A,{}", value2).to_string();

        value2 = value2.overflowing_add(self.registers.is_flag_c() as u8).0;

        let result = self.alu.sub_n(&mut self.registers, value1, value2);

        self.registers.write_byte(&ByteRegister::A, result);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Rotates A right through carry flag.
     */
    fn rra(&mut self) {
        self.last_executed_instruction = "RRA".to_string();
        let carry = self.registers.is_flag_c();

        let value = self.registers.read_byte(&ByteRegister::A);

        let new_carry = value & 0x1 == 1;
        let mut new_a = value >> 1;

        if carry {
            new_a |= 0b10000000;
        } else {
            new_a &= 0b01111111;
        }

        self.registers.write_byte(&ByteRegister::A, new_a);

        self.registers.write_byte(&ByteRegister::F, 0);
        self.registers.set_flag_c(new_carry);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn xor_r(&mut self, register: ByteRegister) {
        self.last_executed_instruction = format!("XOR {}", register.to_string()).to_string();

        let value1 = self.registers.read_byte(&register);
        let mut result = self.registers.read_byte(&ByteRegister::A);

        result = value1 ^ result;

        self.registers.set_flag_z(result == 0);
        self.registers.set_flag_n(false);
        self.registers.set_flag_h(false);
        self.registers.set_flag_c(false);

        self.registers.write_byte(&ByteRegister::A, result);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn xor_n(&mut self) {
        let memory = self.memory.read().unwrap();

        let value: u8 = memory.read_8(self.registers.pc + 1);
        self.last_executed_instruction = format!("XOR {:X}", value).to_string();

        let result = value ^ self.registers.read_byte(&ByteRegister::A);

        self.registers.set_flag_z(result == 0);
        self.registers.set_flag_n(false);
        self.registers.set_flag_h(false);
        self.registers.set_flag_c(false);

        self.registers.write_byte(&ByteRegister::A, result);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /**
     * XORs value in memory address HL with register A. Saves result in A. Sets flag Z if result is 0, resets N, H and C.
     */
    fn xor_mhl(&mut self) {
        self.last_executed_instruction = "XOR (HL)".to_string();

        let memory = self.memory.read().unwrap();

        let mut value = memory.read_8(self.registers.read_word(&WordRegister::HL));
        value = value ^ self.registers.read_byte(&ByteRegister::A);

        self.registers.set_flag_z(value == 0);
        self.registers.set_flag_n(false);
        self.registers.set_flag_h(false);
        self.registers.set_flag_c(false);

        self.registers.write_byte(&ByteRegister::A, value);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn or_r(&mut self, register: ByteRegister) {
        self.last_executed_instruction = format!("OR {}", register.to_string()).to_string();

        let value1: u8 = self.registers.read_byte(&register);
        let value2: u8 = self.registers.read_byte(&ByteRegister::A);

        let result: u8 = self.alu.or_n(&mut self.registers, value1, value2);

        self.registers.write_byte(&ByteRegister::A, result);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn or_mhl(&mut self) {
        self.last_executed_instruction = "OR (HL)".to_string();

        let memory = self.memory.read().unwrap();
        let value1: u8 = memory.read_8(self.registers.read_word(&WordRegister::HL));
        let value2: u8 = self.registers.read_byte(&ByteRegister::A);

        let result: u8 = self.alu.or_n(&mut self.registers, value1, value2);

        self.registers.write_byte(&ByteRegister::A, result);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn or_n(&mut self) {
        let memory = self.memory.read().unwrap();

        let value1: u8 = memory.read_8(self.registers.pc + 1);
        let value2: u8 = self.registers.read_byte(&ByteRegister::A);

        self.last_executed_instruction = format!("OR {:X}", value1).to_string();

        let result: u8 = self.alu.or_n(&mut self.registers, value1, value2);

        self.registers.write_byte(&ByteRegister::A, result);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn and_r(&mut self, register: ByteRegister) {
        let value1: u8 = self.registers.read_byte(&register);
        let value2: u8 = self.registers.read_byte(&ByteRegister::A);

        self.last_executed_instruction = format!("AND {}", register.to_string()).to_string();

        let result: u8 = self.alu.and_n(&mut self.registers, value1, value2);

        self.registers.write_byte(&ByteRegister::A, result);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn and_mhl(&mut self) {
        let memory = self.memory.read().unwrap();

        let value1: u8 = memory.read_8(self.registers.read_word(&WordRegister::HL));
        let value2: u8 = self.registers.read_byte(&ByteRegister::A);

        self.last_executed_instruction = format!("AND {:X}", value1).to_string();

        let result: u8 = self.alu.and_n(&mut self.registers, value1, value2);

        self.registers.write_byte(&ByteRegister::A, result);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn and_n(&mut self) {
        let memory = self.memory.read().unwrap();

        let value1: u8 = memory.read_8(self.registers.pc + 1);
        let value2: u8 = self.registers.a;

        self.last_executed_instruction = format!("AND {:X}", value1).to_string();

        let result: u8 = self.alu.and_n(&mut self.registers, value1, value2);

        self.registers.a = result;

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn cpl(&mut self) {
        self.last_executed_instruction = "CPL".to_string();

        self.registers.set_flag_n(true);
        self.registers.set_flag_h(true);

        let value = !self.registers.read_byte(&ByteRegister::A);
        self.registers.write_byte(&ByteRegister::A, value);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    // --- FLAG INSTRUCTIONS -------------------------------------------------------------------------------------------------------------

    fn scf(&mut self) {
        self.last_executed_instruction = "SCF".to_string();

        self.registers.set_flag_n(false);
        self.registers.set_flag_h(false);
        self.registers.set_flag_c(true);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn ccf(&mut self) {
        self.last_executed_instruction = "CCF".to_string();

        self.registers.set_flag_n(false);
        self.registers.set_flag_h(false);
        self.registers.set_flag_c(!self.registers.is_flag_c());

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn cp_r(&mut self, register: ByteRegister) {
        let n: u8 = self.registers.read_byte(&register);
        let a: u8 = self.registers.read_byte(&ByteRegister::A);

        self.last_executed_instruction = format!("CP {}", register.to_string()).to_string();

        self.alu.cp_n(&mut self.registers, a, n);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn cp_n(&mut self) {
        let memory = self.memory.read().unwrap();

        let n: u8 = memory.read_8(self.registers.pc + 1);
        let a: u8 = self.registers.a;

        self.last_executed_instruction = format!("CP {:X}", n).to_string();

        self.alu.cp_n(&mut self.registers, a, n);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn cp_mhl(&mut self) {
        let memory = self.memory.read().unwrap();

        let n: u8 = memory.read_8(self.registers.read_word(&WordRegister::HL));
        let a: u8 = self.registers.a;

        self.last_executed_instruction = "CP (HL)".to_string();

        self.alu.cp_n(&mut self.registers, a, n);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn ld_r_n(&mut self, register: ByteRegister) {
        let memory = self.memory.read().unwrap();

        let value = memory.read_8(self.registers.read_word(&WordRegister::PC) + 1);
        self.registers.write_byte(&register, value);

        self.last_executed_instruction =
            format!("LD {},{:X}", register.to_string(), value).to_string();

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn ld_r_r(&mut self, register_to: ByteRegister, register_from: ByteRegister) {
        let value = self.registers.read_byte(&register_from);
        self.registers.write_byte(&register_to, value);

        self.last_executed_instruction = format!(
            "LD {},{}",
            register_to.to_string(),
            register_from.to_string()
        )
        .to_string();

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn ld_hl_sp_n(&mut self) {
        let memory = self.memory.read().unwrap();

        let add1 = self.registers.read_word(&WordRegister::SP);
        let add2 = memory.read_8_signed(self.registers.read_word(&WordRegister::PC) + 1);

        let new_value = self
            .alu
            .add_nn_signed(&mut self.registers, add1, add2 as i16);
        self.registers.write_word(&WordRegister::HL, new_value);

        self.last_executed_instruction = format!("LD HL,SP + {:X}", add2).to_string();

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 12;
    }

    fn ld_sp_hl(&mut self) {
        self.last_executed_instruction = "LD SP,HL".to_string();

        let new_value = self.registers.read_word(&WordRegister::HL);
        self.registers.write_word(&WordRegister::SP, new_value);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn ld_r_mrr(
        &mut self,
        register_to: ByteRegister,
        register_from: WordRegister,
    ) {
        self.last_executed_instruction = format!(
            "LD {},({})",
            register_to.to_string(),
            register_from.to_string()
        )
        .to_string();

        let memory = self.memory.read().unwrap();
        let value = memory.read_8(self.registers.read_word(&register_from));
        self.registers.write_byte(&register_to, value);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Writes value from register A to memory address $FF00 + n.
     */
    fn ldh_n_a(&mut self) {
        let mut memory = self.memory.write().unwrap();
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
    fn ld_mc_a(&mut self) {
        let mut memory = self.memory.write().unwrap();

        self.last_executed_instruction = "LD ($FF00 + C),A".to_string();

        let mem_addr: u16 = 0xFF00 + self.registers.read_byte(&ByteRegister::C) as u16;
        memory.write_8(mem_addr, self.registers.a);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Writes value from memory address $FF00 + C to register A.
     */
    fn ld_a_mc(&mut self) {
        self.last_executed_instruction = "LD A, ($FF00 + C)".to_string();

        let mem_addr: u16 = 0xFF00 + self.registers.read_byte(&ByteRegister::C) as u16;

        let memory = self.memory.read().unwrap();
        self.registers.a = memory.read_8(mem_addr);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Writes value from memory address $FF00 + n to register A.
     */
    fn ldh_a_n(&mut self) {
        let memory = self.memory.read().unwrap();
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
    fn ld_nn_a(&mut self) {
        let mut memory = self.memory.write().unwrap();
        let mem_addr: u16 = memory.read_16(self.registers.pc + 1);

        self.last_executed_instruction = format!("LD ({:X}),A", mem_addr).to_string();

        memory.write_8(mem_addr, self.registers.a);

        self.pc_to_increment = 3;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Writes value from memory address nn to register A.
     */
    fn ld_a_nn(&mut self) {
        let memory = self.memory.read().unwrap();
        let mem_addr: u16 = memory.read_16(self.registers.pc + 1);

        self.last_executed_instruction = format!("LD A, ({:X})", mem_addr).to_string();

        self.registers.a = memory.read_8(mem_addr);

        self.pc_to_increment = 3;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Writes value from register A to memory address contained in HL and decreases HL.
     */
    fn ldd_mhl_a(&mut self) {
        let mut memory = self.memory.write().unwrap();
        self.last_executed_instruction = "LDD (HL),A".to_string();

        memory.write_8(
            self.registers.read_word(&WordRegister::HL),
            self.registers.a,
        );

        let value: u16 = self.registers.read_word(&WordRegister::HL);
        self.registers
            .write_word(&WordRegister::HL, self.alu.dec_nn(value));

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    /**
     * Writes value from register A to memory address contained in HL and increases HL.
     */
    fn ldi_mhl_a(&mut self) {
        let mut memory = self.memory.write().unwrap();
        self.last_executed_instruction = "LDI (HL),A".to_string();

        memory.write_8(
            self.registers.read_word(&WordRegister::HL),
            self.registers.a,
        );

        let value: u16 = self.registers.read_word(&WordRegister::HL);
        self.registers
            .write_word(&WordRegister::HL, self.alu.inc_nn(value));

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn ld_mrr_r(
        &mut self,
        register_to: WordRegister,
        register_from: ByteRegister,
    ) {
        let mut memory = self.memory.write().unwrap();

        self.last_executed_instruction = format!(
            "LD ({}),{}",
            register_to.to_string(),
            register_from.to_string()
        )
        .to_string();

        memory.write_8(
            self.registers.read_word(&register_to),
            self.registers.read_byte(&register_from),
        );

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Writes 8bit value to memory address contained in HL.
     */
    fn ld_mhl_n(&mut self) {
        let mut memory = self.memory.write().unwrap();

        let value = memory.read_8(self.registers.pc + 1);

        self.last_executed_instruction = format!("LD (HL),{:X}", value).to_string();

        memory.write_8(self.registers.read_word(&WordRegister::HL), value);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 12;
    }

    fn ldi_a_mhl(&mut self) {
        self.last_executed_instruction = "LDI A,(HL)".to_string();

        let mut new_value_hl = self.registers.read_word(&WordRegister::HL);

        let memory = self.memory.read().unwrap();
        let value: u8 = memory.read_8(new_value_hl);
        self.registers.a = value;

        new_value_hl = self.alu.inc_nn(new_value_hl);

        self.registers.write_word(&WordRegister::HL, new_value_hl);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn ldd_a_mhl(&mut self) {
        let memory = self.memory.read().unwrap();

        self.last_executed_instruction = "LDD A,(HL)".to_string();

        let mut new_value_hl = self.registers.read_word(&WordRegister::HL);
        let value: u8 = memory.read_8(new_value_hl);
        self.registers.a = value;

        new_value_hl = self.alu.dec_nn(new_value_hl);

        self.registers.write_word(&WordRegister::HL, new_value_hl);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 8;
    }

    fn ld_rr_nn(&mut self, register: WordRegister) {
        let memory = self.memory.read().unwrap();

        let value: u16 = memory.read_16(self.registers.pc + 1);
        self.registers.write_word(&register, value);

        self.last_executed_instruction = format!(
            "LD {},{:X}",
            register.to_string(),
            self.registers.read_word(&register)
        )
        .to_string();

        self.pc_to_increment = 3;
        self.last_instruction_ccycles = 12;
    }

    fn ld_mnn_sp(&mut self) {
        let mut memory = self.memory.write().unwrap();
        let mem_addr = memory.read_16(self.registers.read_word(&WordRegister::PC) + 1);

        self.last_executed_instruction = format!("LD ({:X}),SP", mem_addr).to_string();

        let value = self.registers.read_word(&WordRegister::SP);
        memory.write_16(mem_addr, value);

        self.pc_to_increment = 3;
        self.last_instruction_ccycles = 20;
    }

    // --- JUMP INSTRUCTIONS ----------------------------------------------------------------------------------------------------------------

    /**
     * Jumps to the current PC + n
     */
    fn jr_n(&mut self) {
        let memory = self.memory.read().unwrap();

        let to_sum = memory.read_8_signed(self.registers.pc + 1) + 2;

        self.registers.pc = self.registers.pc.overflowing_add(to_sum as u16).0;

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 12;

        self.last_executed_instruction = format!("JR {:X}", self.registers.pc).to_string();
    }

    /**
     * Jumps to the current PC + n only if the flag Z is not set. Otherwise, continues to the next instruction.
     */
    fn jr_nz_n(&mut self) {
        let memory = self.memory.read().unwrap();

        let possible_value: i8 = memory.read_8_signed(self.registers.pc + 1);

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
    fn jr_z_n(&mut self) {
        let memory = self.memory.read().unwrap();
        let possible_value: i8 = memory.read_8_signed(self.registers.pc + 1);

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
    fn jr_c_n(&mut self) {
        let memory = self.memory.read().unwrap();
        let possible_value: i8 = memory.read_8_signed(self.registers.pc + 1);

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
    fn jr_nc_n(&mut self) {
        let memory = self.memory.read().unwrap();
        let possible_value: i8 = memory.read_8_signed(self.registers.pc + 1);

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
    fn jp_nn(&mut self) {
        let memory = self.memory.read().unwrap();
        self.registers.pc = memory.read_16(self.registers.pc + 1);

        self.last_executed_instruction = format!("JP {:X}", self.registers.pc).to_string();

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Jumps to the 16 bit address contained in HL.
     */
    fn jp_mhl(&mut self) {
        self.registers.pc = self.registers.read_word(&WordRegister::HL);

        self.last_executed_instruction = "JP (HL)".to_string();

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 4;
    }

    fn jp_c_nn(&mut self) {
        let memory = self.memory.read().unwrap();
        let possible_value = memory.read_16(self.registers.pc + 1);

        self.last_executed_instruction = format!("JP C,{:X}", possible_value).to_string();

        self.registers.pc += 3;

        if self.registers.is_flag_c() {
            self.registers.pc = possible_value;
            self.last_instruction_ccycles = 16;
        } else {
            self.last_instruction_ccycles = 12;
        }

        self.pc_to_increment = 0;
    }

    fn jp_nc_nn(&mut self) {
        let memory = self.memory.read().unwrap();
        let possible_value = memory.read_16(self.registers.pc + 1);

        self.last_executed_instruction = format!("JP NC,{:X}", possible_value).to_string();

        self.registers.pc += 3;

        if self.registers.is_flag_c() {
            self.registers.pc = possible_value;
            self.last_instruction_ccycles = 16;
        } else {
            self.last_instruction_ccycles = 12;
        }

        self.pc_to_increment = 0;
    }

    /**
     * Jumps to the indicated address only if the flag Z is set. Otherwise, continues to the next instruction.
     */
    fn jp_z_nn(&mut self) {
        let memory = self.memory.read().unwrap();
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
    fn jp_nz_nn(&mut self) {
        let memory = self.memory.read().unwrap();
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
    fn call(&mut self) {
        let next_pc: u16 = self.registers.pc + 3;
        self.push_vv(next_pc);

        let memory = self.memory.read().unwrap();
        self.registers.pc = memory.read_16(self.registers.pc + 1);

        self.last_executed_instruction = format!("CALL {:X}", self.registers.pc).to_string();

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 24;
    }

    /**
     * If flag Z is set, push address of next instruction onto stack and then jump to address nn.
     */
    fn call_z_nn(&mut self) {
        self.last_executed_instruction = format!("CALL Z,{:X}", self.registers.pc).to_string();

        if !self.registers.is_flag_z() {
            self.pc_to_increment = 3;
            self.last_instruction_ccycles = 12;
        }

        let next_pc: u16 = self.registers.pc + 3;
        self.push_vv(next_pc);

        let memory = self.memory.read().unwrap();
        self.registers.pc = memory.read_16(self.registers.pc + 1);

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 24;
    }

    /**
     * If flag Z is reset, push address of next instruction onto stack and then jump to address nn.
     */
    fn call_nz_nn(&mut self) {
        self.last_executed_instruction = format!("CALL NZ,{:X}", self.registers.pc).to_string();

        if self.registers.is_flag_z() {
            self.pc_to_increment = 3;
            self.last_instruction_ccycles = 12;
        }

        let next_pc: u16 = self.registers.pc + 3;
        self.push_vv(next_pc);

        let memory = self.memory.read().unwrap();
        self.registers.pc = memory.read_16(self.registers.pc + 1);

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 24;
    }

    /**
     * If flag Z is set, push address of next instruction onto stack and then jump to address nn.
     */
    fn call_c_nn(&mut self) {
        self.last_executed_instruction = format!("CALL C,{:X}", self.registers.pc).to_string();

        if !self.registers.is_flag_c() {
            self.pc_to_increment = 3;
            self.last_instruction_ccycles = 12;
        }

        let next_pc: u16 = self.registers.pc + 3;
        self.push_vv(next_pc);

        let memory = self.memory.read().unwrap();
        self.registers.pc = memory.read_16(self.registers.pc + 1);

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 24;
    }

    /**
     * If flag C is reset, push address of next instruction onto stack and then jump to address nn.
     */
    fn call_nc_nn(&mut self) {
        self.last_executed_instruction = format!("CALL NC,{:X}", self.registers.pc).to_string();

        if self.registers.is_flag_c() {
            self.pc_to_increment = 3;
            self.last_instruction_ccycles = 12;
        }

        let next_pc: u16 = self.registers.pc + 3;
        self.push_vv(next_pc);

        let memory = self.memory.read().unwrap();
        self.registers.pc = memory.read_16(self.registers.pc + 1);

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 24;
    }

    /**
     * Pop two bytes from stack & jump to that address.
     */
    fn ret(&mut self) {
        self.last_executed_instruction = "RET".to_string();

        self.registers.pc = self.pop_vv();

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Pop two bytes from stack & jump to that address, enabling interruptions.
     */
    fn reti(&mut self) {
        self.last_executed_instruction = "RETI".to_string();

        self.registers.pc = self.pop_vv();

        self.ime = true;

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 16;
    }

    /**
     * Pop two bytes from stack & jump to that address if flag Z is not set.
     */
    fn ret_nz(&mut self) {
        self.last_executed_instruction = "RET NZ".to_string();

        if !self.registers.is_flag_z() {
            self.registers.pc = self.pop_vv();
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
    fn ret_z(&mut self) {
        self.last_executed_instruction = "RET Z".to_string();

        if self.registers.is_flag_z() {
            self.registers.pc = self.pop_vv();
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
    fn ret_nc(&mut self) {
        self.last_executed_instruction = "RET NC".to_string();

        if !self.registers.is_flag_c() {
            self.registers.pc = self.pop_vv();
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
    fn ret_c(&mut self) {
        self.last_executed_instruction = "RET C".to_string();

        if self.registers.is_flag_c() {
            self.registers.pc = self.pop_vv();
            self.last_instruction_ccycles = 20;
        } else {
            self.registers.pc += 1;
            self.last_instruction_ccycles = 8;
        }

        self.pc_to_increment = 0;
    }

    // --- RESTART INSTRUCTIONS ------------------------------------------------------------------------------------------------------------

    fn rst_v_w_out(&mut self, value: u8) {
        self.last_executed_instruction = format!("RST ${:X}", value).to_string();
        self.rst_v(value)
    }

    // --- STACK INSTRUCTIONS --------------------------------------------------------------------------------------------------------------

    fn push_rr(&mut self, register: WordRegister) {
        self.last_executed_instruction = format!("PUSH {}", register.to_string()).to_string();

        let reg: u16 = self.registers.read_word(&register);
        self.push_vv(reg);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 16;
    }

    fn pop_rr(&mut self, register: WordRegister) {
        self.last_executed_instruction = format!("POP {}", register.to_string()).to_string();

        let popped: u16 = self.pop_vv();
        self.registers.write_word(&register, popped);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 12;
    }

    fn rr_r(&mut self, register: ByteRegister) {
        self.last_executed_instruction = format!("RR {}", register.to_string()).to_string();

        let mut value = self.registers.read_byte(&register);

        let carry: bool = value & 0b1 == 1;
        let msf: u8 = if self.registers.is_flag_c() {
            0b10000000
        } else {
            0
        };

        value = msf | ((value >> 1) & 0b01111111);

        self.registers.set_flag_z(value == 0);
        self.registers.set_flag_c(carry);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.registers.write_byte(&register, value);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn rl_r(&mut self, register: ByteRegister) {
        self.last_executed_instruction = format!("RL {}", register.to_string()).to_string();

        let value = self.registers.read_byte(&register);
        let new_carry: bool = value & 0b10000000 == 0b10000000;

        self.registers.write_byte(
            &register,
            (value << 1) | (0b00000001 & (self.registers.is_flag_c() as u8)),
        );

        self.registers.set_flag_z(value == 0);
        self.registers.set_flag_c(new_carry);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    /**
     * Rotate left through carry register A.
     */
    fn rla(&mut self) {
        self.last_executed_instruction = "RLA".to_string();
        let new_carry: bool = self.registers.a & 0b10000000 == 0b10000000;

        self.registers.a = self.registers.a << 1;
        self.registers.a = self.registers.a | (0b00000001 & (self.registers.is_flag_c() as u8));

        self.registers.set_flag_z(false);
        self.registers.set_flag_c(new_carry);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn rlc_r(&mut self, register: ByteRegister) {
        self.last_executed_instruction = format!("RLC {}", register.to_string()).to_string();

        let mut value = self.registers.read_byte(&register);
        let new_carry: bool = value & 0b10000000 == 0b10000000;

        value = value << 1;
        value = value | (new_carry as u8);

        self.registers.set_flag_z(value == 0);
        self.registers.set_flag_n(false);
        self.registers.set_flag_h(false);
        self.registers.set_flag_c(new_carry);

        self.registers.write_byte(&register, value);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn rlc_mrr(&mut self, register: WordRegister) {
        let mut memory = self.memory.write().unwrap();

        self.last_executed_instruction = format!("RLC ({})", register.to_string()).to_string();

        let address = self.registers.read_word(&register);
        let mut value = memory.read_8(address);
        let new_carry: bool = value & 0b10000000 == 0b10000000;

        value = value << 1;
        value = value | (new_carry as u8);

        self.registers.set_flag_z(value == 0);
        self.registers.set_flag_n(false);
        self.registers.set_flag_h(false);
        self.registers.set_flag_c(new_carry);

        memory.write_8(address, value);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 16;
    }

    fn rlca(&mut self) {
        self.last_executed_instruction = "RLCA".to_string();

        let mut value = self.registers.read_byte(&ByteRegister::A);
        let new_carry: bool = value & 0b10000000 == 0b10000000;

        value = value << 1;
        value = value | (new_carry as u8);

        self.registers.set_flag_z(false);
        self.registers.set_flag_c(new_carry);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.registers.write_byte(&ByteRegister::A, value);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    fn srl_r(&mut self, register: ByteRegister) {
        self.last_executed_instruction = format!("SRL {}", register.to_string()).to_string();

        let value = self.registers.read_byte(&register);

        let carry: bool = value & 0b1 == 1;
        let msf: u8 = value & 0b10000000;

        let result = msf | ((value >> 1) & 0b01111111);
        self.registers.write_byte(&register, result);

        self.registers.set_flag_z(result == 0);
        self.registers.set_flag_c(carry);
        self.registers.set_flag_h(false);
        self.registers.set_flag_n(false);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn sla_r(&mut self, register: ByteRegister) {
        self.last_executed_instruction = format!("SLA {}", register.to_string()).to_string();

        let mut value = self.registers.read_byte(&register);

        let carry: bool = value & 0b10000000 == 0b10000000;

        value = value << 1;

        self.registers.write_byte(&register, value);

        self.registers.set_flag_z(value == 0);
        self.registers.set_flag_n(false);
        self.registers.set_flag_h(false);
        self.registers.set_flag_c(carry);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn swap_r(&mut self, register: ByteRegister) {
        self.last_executed_instruction = format!("SWAP {}", register.to_string()).to_string();

        let mut value = self.registers.read_byte(&register);
        value = self.alu.swap_n(&mut self.registers, value);
        self.registers.write_byte(&register, value);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn res_v_r(&mut self, bit: u8, register: ByteRegister) {
        self.last_executed_instruction =
            format!("RES {},{}", bit, register.to_string()).to_string();

        let mask = !(0x1 << bit);

        let mut value = self.registers.read_byte(&register) & mask;
        self.registers.write_byte(&register, value);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn res_v_mhl(&mut self, bit: u8) {
        let mut memory = self.memory.write().unwrap();
        self.last_executed_instruction = format!("RES {},(HL)", bit).to_string();

        let mut value = memory.read_8(self.registers.read_word(&WordRegister::HL));
        value &= !(0x1 << bit);
        memory.write_8(self.registers.read_word(&WordRegister::HL), value);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 16;
    }

    fn set_v_r(&mut self, bit: u8, register: ByteRegister) {
        self.last_executed_instruction =
            format!("SET {},{}", bit, register.to_string()).to_string();

        let mask = 0x1 << bit;

        let mut value = self.registers.read_byte(&register) | mask;
        self.registers.write_byte(&register, value);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn set_v_mhl(&mut self, bit: u8) {
        let mut memory = self.memory.write().unwrap();
        self.last_executed_instruction = format!("SET {},(HL)", bit).to_string();

        let mut value = memory.read_8(self.registers.read_word(&WordRegister::HL));
        value |= 0x1 << bit;
        memory.write_8(self.registers.read_word(&WordRegister::HL), value);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 16;
    }

    fn daa(&mut self) {
        self.last_executed_instruction = "DAA".to_string();

        let mut register_a = Wrapping(self.registers.read_byte(&ByteRegister::A));

        if !self.registers.is_flag_n() {
            // Addition
            if self.registers.is_flag_c() || register_a.0 > 0x99 {
                register_a += Wrapping(0x60);
                self.registers.set_flag_c(true);
            }

            if self.registers.is_flag_h() || (register_a.0 & 0x0f) > 0x09 {
                register_a += Wrapping(0x6);
            }
        } else {
            if self.registers.is_flag_c() {
                register_a -= Wrapping(0x60);
            }

            if self.registers.is_flag_h() {
                register_a -= Wrapping(0x6);
            }
        }

        self.registers.set_flag_z(register_a.0 == 0);
        self.registers.set_flag_h(false);

        self.registers.write_byte(&ByteRegister::A, register_a.0);

        self.pc_to_increment = 1;
        self.last_instruction_ccycles = 4;
    }

    // --- INTERNAL --------------------------------------------------------------------------------

    fn push_vv(&mut self, value: u16) {
        let mut memory = self.memory.write().unwrap();
        memory.write_16(self.registers.sp - 2, value);
        self.registers.sp = self.registers.sp - 2;
    }

    fn pop_vv(&mut self) -> u16 {
        let memory = self.memory.read().unwrap();
        let value = memory.read_16(self.registers.sp);
        self.registers.sp += 2;

        return value;
    }

    fn rst_v(&mut self, value: u8) {
        self.push_vv(self.registers.pc + 1);

        self.registers.pc = value as u16;

        self.pc_to_increment = 0;
        self.last_instruction_ccycles = 16;
    }

    fn interrupt_vv(&mut self, new_address: u16) {
        self.ime = false;
        self.push_vv(self.registers.pc);
        self.registers.pc = new_address;
    }

    fn bit_v_r(&mut self, bit: u8, register: ByteRegister) {
        self.last_executed_instruction =
            format!("BIT {},{}", bit, register.to_string()).to_string();

        let mask = 1u8 << bit;
        let value = self.registers.read_byte(&register);

        let zero = value & mask != mask;

        self.registers.set_flag_z(zero);
        self.registers.set_flag_n(false);
        self.registers.set_flag_h(true);

        self.pc_to_increment = 2;
        self.last_instruction_ccycles = 8;
    }

    fn bit_v_mhl(&mut self, bit: u8) {
        self.last_executed_instruction = format!("BIT {},(HL)", bit).to_string();

        let mask = 1u8 << bit;

        let memory = self.memory.read().unwrap();
        let value = memory.read_8(self.registers.read_word(&WordRegister::HL));

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

    pub fn vblank_interrupt(&mut self) {
        {
            self.memory.write().unwrap().interrupt_flag().set_vblank(false);
        }

        self.interrupt_vv(0x40)
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

#[cfg(test)]
mod test {
    use crate::cpu::cpu::CPU;
    use crate::cpu::registers::{ByteRegister, WordRegister};
    use std::sync::{Arc, RwLock};
    use crate::memory::memory::Memory;

    #[test]
    fn test_inc_rr() {
        let registers = [
            WordRegister::BC,
            WordRegister::DE,
            WordRegister::HL,
            WordRegister::SP,
        ];

        let mut cpu = CPU::new(Arc::new(RwLock::new(Memory::default())), false, false);

        for register in registers.iter() {
            // 0x0000 to 0x0001
            cpu.registers.write_word(register, 0x0000);
            let old_f = cpu.registers.read_byte(&ByteRegister::F);

            cpu.inc_rr(*register);
            assert_eq!(cpu.registers.read_word(register), 0x0001);
            assert_eq!(old_f, cpu.registers.read_byte(&ByteRegister::F));

            // 0xFFFF to 0x0000
            cpu.registers.write_word(register, 0xFFFF);
            let old_f = cpu.registers.read_byte(&ByteRegister::F);

            cpu.inc_rr(*register);
            assert_eq!(cpu.registers.read_word(register), 0x0000);
            assert_eq!(old_f, cpu.registers.read_byte(&ByteRegister::F));
        }
    }

    #[test]
    fn test_dec_rr() {
        let registers = [
            WordRegister::BC,
            WordRegister::DE,
            WordRegister::HL,
            WordRegister::SP,
        ];
        let mut cpu = CPU::new(Arc::new(RwLock::new(Memory::default())), false, false);

        for register in registers.iter() {
            // 0x0001 to 0x0000
            cpu.registers.write_word(register, 0x0001);
            let old_f = cpu.registers.read_byte(&ByteRegister::F);

            cpu.dec_rr(*register);
            assert_eq!(cpu.registers.read_word(register), 0x0000);
            assert_eq!(old_f, cpu.registers.read_byte(&ByteRegister::F));

            // 0x0000 to 0xFFFF
            cpu.registers.write_word(register, 0x0000);
            let old_f = cpu.registers.read_byte(&ByteRegister::F);

            cpu.dec_rr(*register);
            assert_eq!(cpu.registers.read_word(register), 0xFFFF);
            assert_eq!(old_f, cpu.registers.read_byte(&ByteRegister::F));
        }
    }

    fn test_add_hl_rr() {
        let registers = [
            WordRegister::BC,
            WordRegister::DE,
            WordRegister::HL,
            WordRegister::SP,
        ];
        let mut cpu = CPU::new(Arc::new(RwLock::new(Memory::default())), false, false);

        for register in registers.iter() {
            cpu.registers.write_word(&WordRegister::HL, 0xFFFF);
            cpu.registers.write_word(register, 0x0001);
            cpu.registers.write_byte(&ByteRegister::F, 0xFF);

            let old_flag_z = cpu.registers.is_flag_z();

            cpu.add_hl_rr(*register);

            assert_eq!(cpu.registers.read_word(&WordRegister::HL), 0x0000);
            assert_eq!(old_flag_z, cpu.registers.is_flag_z());
            assert_eq!(false, cpu.registers.is_flag_n());
            assert_eq!(true, cpu.registers.is_flag_h());
            assert_eq!(true, cpu.registers.is_flag_c());

            /////////

            cpu.registers.write_word(&WordRegister::HL, 0x0000);
            cpu.registers.write_word(register, 0x0001);
            cpu.registers.write_byte(&ByteRegister::F, 0xFF);

            let old_flag_z = cpu.registers.is_flag_z();

            cpu.add_hl_rr(*register);

            assert_eq!(cpu.registers.read_word(&WordRegister::HL), 0x0001);
            assert_eq!(old_flag_z, cpu.registers.is_flag_z());
            assert_eq!(false, cpu.registers.is_flag_n());
            assert_eq!(false, cpu.registers.is_flag_h());
            assert_eq!(false, cpu.registers.is_flag_c());

            /////////

            cpu.registers.write_word(&WordRegister::HL, 0x0FFF);
            cpu.registers.write_word(register, 0x0001);
            cpu.registers.write_byte(&ByteRegister::F, 0xFF);

            let old_flag_z = cpu.registers.is_flag_z();

            cpu.add_hl_rr(*register);

            assert_eq!(cpu.registers.read_word(&WordRegister::HL), 0x1000);
            assert_eq!(old_flag_z, cpu.registers.is_flag_z());
            assert_eq!(false, cpu.registers.is_flag_n());
            assert_eq!(true, cpu.registers.is_flag_h());
            assert_eq!(false, cpu.registers.is_flag_c());

            /////////

            cpu.registers.write_word(&WordRegister::HL, 0x1000);
            cpu.registers.write_word(register, 0x0001);
            cpu.registers.write_byte(&ByteRegister::F, 0xFF);

            let old_flag_z = cpu.registers.is_flag_z();

            cpu.add_hl_rr(*register);

            assert_eq!(cpu.registers.read_word(&WordRegister::HL), 0x1001);
            assert_eq!(old_flag_z, cpu.registers.is_flag_z());
            assert_eq!(false, cpu.registers.is_flag_n());
            assert_eq!(false, cpu.registers.is_flag_h());
            assert_eq!(false, cpu.registers.is_flag_c());
        }
    }
}
