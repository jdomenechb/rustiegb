mod cpu;
mod memory;
mod math;

use cpu::cpu::CPU;
use memory::memory::Memory;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    // Setting up CPU
    let mut cpu = CPU::new();

    // Setting up memory
    let mut memory = Memory::new("./cpu_instrs.gb");

    // Main loop
    loop {
        let instruction: &u8 = &memory.read_8(cpu.registers.pc);

        print!("{:X}: ", cpu.registers.pc);

        match instruction {
            0x00 => cpu.nop(),
            0x01 => cpu.ld_bc_nn(&memory),
            0x03 => cpu.inc_bc(),
            0x05 => cpu.dec_b(),
            0x06 => cpu.ld_b_n(&memory),
            0x0C => cpu.inc_c(),
            0x0E => cpu.ld_c_n(&memory),
            0x18 => cpu.jr_n(&memory),
            0x1E => cpu.ld_e_n(&memory),
            0x1F => cpu.rra(),
            0x20 => cpu.jr_nz_n(&memory),
            0x21 => cpu.ld_hl_nn(&memory),
            0x23 => cpu.inc_hl(),
            0x28 => cpu.jr_z_n(&memory),
            0x2A => cpu.ldi_a_mhl(&memory),
            0x30 => cpu.jr_nc_n(&memory),
            0x31 => cpu.ld_sp_nn(&memory),
            0x32 => cpu.ldd_hl_a(&mut memory),
            0x38 => cpu.jr_c_n(&memory),
            0x3C => cpu.inc_a(),
            0x3E => cpu.ld_a_n(&memory),
            0x4E => cpu.ld_c_mhl(&memory),
            0x49 => cpu.ld_c_c(),
            0x60 => cpu.ld_h_b(),
            0x66 => cpu.ld_h_mhl(&memory),
            0x6E => cpu.ld_l_mhl(&memory),
            0x78 => cpu.ld_a_b(),
            0x7C => cpu.ld_a_h(),
            0x7D => cpu.ld_a_l(),
            0x7E => cpu.ld_a_mhl(&memory),
            0xAF => cpu.xor_a(),
            0xB1 => cpu.or_c(),
            0xC3 => cpu.jp_nn(&memory),
            0xC5 => cpu.push_bc(&mut memory),
            0xC6 => cpu.add_a_n(&memory),
            0xC9 => cpu.ret(&mut memory),
            0xCD => cpu.call(&mut memory),
            0xE0 => cpu.ldh_n_a(&mut memory),
            0xE1 => cpu.pop_hl(&mut memory),
            0xE5 => cpu.push_hl(&mut memory),
            0xEA => cpu.ld_nn_a(&mut memory),
            0xF0 => cpu.ldh_a_n(&mut memory),
            0xF1 => cpu.pop_af(&mut memory),
            0xF3 => cpu.di(),
            0xF5 => cpu.push_af(&mut memory),
            0xFE => cpu.cp_n(&memory),
            0xFF => cpu.rst_38(&mut memory),
            _ => {
                println!("Instruction not implemented: {:X}", instruction);
                println!("{:#X?}", cpu);
                break;
            }
        }

        //sleep(Duration::from_millis(300));
    }
}
