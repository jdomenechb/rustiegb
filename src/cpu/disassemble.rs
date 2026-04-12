use crate::Word;
use crate::memory::Memory;

pub(super) fn disassemble(pc: Word, memory: &Memory) -> String {
    let op = memory.read_byte(pc);
    let b1 = memory.read_byte(pc.wrapping_add(1));
    let b2 = memory.read_byte(pc.wrapping_add(2));
    let nn: Word = (b2 as Word) << 8 | b1 as Word;

    match op {
        0x00 => "NOP".to_string(),
        0x01 => format!("LD BC,${:04X}", nn),
        0x02 => "LD (BC),A".to_string(),
        0x03 => "INC BC".to_string(),
        0x04 => "INC B".to_string(),
        0x05 => "DEC B".to_string(),
        0x06 => format!("LD B,${:02X}", b1),
        0x07 => "RLCA".to_string(),
        0x08 => format!("LD (${:04X}),SP", nn),
        0x09 => "ADD HL,BC".to_string(),
        0x0A => "LD A,(BC)".to_string(),
        0x0B => "DEC BC".to_string(),
        0x0C => "INC C".to_string(),
        0x0D => "DEC C".to_string(),
        0x0E => format!("LD C,${:02X}", b1),
        0x0F => "RRCA".to_string(),

        0x10 => "STOP".to_string(),
        0x11 => format!("LD DE,${:04X}", nn),
        0x12 => "LD (DE),A".to_string(),
        0x13 => "INC DE".to_string(),
        0x14 => "INC D".to_string(),
        0x15 => "DEC D".to_string(),
        0x16 => format!("LD D,${:02X}", b1),
        0x17 => "RLA".to_string(),
        0x18 => format!("JR ${:02X}", b1),
        0x19 => "ADD HL,DE".to_string(),
        0x1A => "LD A,(DE)".to_string(),
        0x1B => "DEC DE".to_string(),
        0x1C => "INC E".to_string(),
        0x1D => "DEC E".to_string(),
        0x1E => format!("LD E,${:02X}", b1),
        0x1F => "RRA".to_string(),

        0x20 => format!("JR NZ,${:02X}", b1),
        0x21 => format!("LD HL,${:04X}", nn),
        0x22 => "LD (HL+),A".to_string(),
        0x23 => "INC HL".to_string(),
        0x24 => "INC H".to_string(),
        0x25 => "DEC H".to_string(),
        0x26 => format!("LD H,${:02X}", b1),
        0x27 => "DAA".to_string(),
        0x28 => format!("JR Z,${:02X}", b1),
        0x29 => "ADD HL,HL".to_string(),
        0x2A => "LD A,(HL+)".to_string(),
        0x2B => "DEC HL".to_string(),
        0x2C => "INC L".to_string(),
        0x2D => "DEC L".to_string(),
        0x2E => format!("LD L,${:02X}", b1),
        0x2F => "CPL".to_string(),

        0x30 => format!("JR NC,${:02X}", b1),
        0x31 => format!("LD SP,${:04X}", nn),
        0x32 => "LD (HL-),A".to_string(),
        0x33 => "INC SP".to_string(),
        0x34 => "INC (HL)".to_string(),
        0x35 => "DEC (HL)".to_string(),
        0x36 => format!("LD (HL),${:02X}", b1),
        0x37 => "SCF".to_string(),
        0x38 => format!("JR C,${:02X}", b1),
        0x39 => "ADD HL,SP".to_string(),
        0x3A => "LD A,(HL-)".to_string(),
        0x3B => "DEC SP".to_string(),
        0x3C => "INC A".to_string(),
        0x3D => "DEC A".to_string(),
        0x3E => format!("LD A,${:02X}", b1),
        0x3F => "CCF".to_string(),

        0x40..=0x7F => {
            if op == 0x76 {
                "HALT".to_string()
            } else {
                format!("LD {},{}", reg_name((op >> 3) & 0x7), reg_name(op & 0x7))
            }
        }

        0x80..=0x87 => format!("ADD A,{}", reg_name(op & 0x7)),
        0x88..=0x8F => format!("ADC A,{}", reg_name(op & 0x7)),
        0x90..=0x97 => format!("SUB {}", reg_name(op & 0x7)),
        0x98..=0x9F => format!("SBC A,{}", reg_name(op & 0x7)),
        0xA0..=0xA7 => format!("AND {}", reg_name(op & 0x7)),
        0xA8..=0xAF => format!("XOR {}", reg_name(op & 0x7)),
        0xB0..=0xB7 => format!("OR {}", reg_name(op & 0x7)),
        0xB8..=0xBF => format!("CP {}", reg_name(op & 0x7)),

        0xC0 => "RET NZ".to_string(),
        0xC1 => "POP BC".to_string(),
        0xC2 => format!("JP NZ,${:04X}", nn),
        0xC3 => format!("JP ${:04X}", nn),
        0xC4 => format!("CALL NZ,${:04X}", nn),
        0xC5 => "PUSH BC".to_string(),
        0xC6 => format!("ADD A,${:02X}", b1),
        0xC7 => "RST $00".to_string(),
        0xC8 => "RET Z".to_string(),
        0xC9 => "RET".to_string(),
        0xCA => format!("JP Z,${:04X}", nn),
        0xCB => disassemble_cb(b1),
        0xCC => format!("CALL Z,${:04X}", nn),
        0xCD => format!("CALL ${:04X}", nn),
        0xCE => format!("ADC A,${:02X}", b1),
        0xCF => "RST $08".to_string(),

        0xD0 => "RET NC".to_string(),
        0xD1 => "POP DE".to_string(),
        0xD2 => format!("JP NC,${:04X}", nn),
        0xD4 => format!("CALL NC,${:04X}", nn),
        0xD5 => "PUSH DE".to_string(),
        0xD6 => format!("SUB ${:02X}", b1),
        0xD7 => "RST $10".to_string(),
        0xD8 => "RET C".to_string(),
        0xD9 => "RETI".to_string(),
        0xDA => format!("JP C,${:04X}", nn),
        0xDC => format!("CALL C,${:04X}", nn),
        0xDE => format!("SBC A,${:02X}", b1),
        0xDF => "RST $18".to_string(),

        0xE0 => format!("LDH ($FF{:02X}),A", b1),
        0xE1 => "POP HL".to_string(),
        0xE2 => "LD ($FF00+C),A".to_string(),
        0xE5 => "PUSH HL".to_string(),
        0xE6 => format!("AND ${:02X}", b1),
        0xE7 => "RST $20".to_string(),
        0xE8 => format!("ADD SP,${:02X}", b1),
        0xE9 => "JP (HL)".to_string(),
        0xEA => format!("LD (${:04X}),A", nn),
        0xEE => format!("XOR ${:02X}", b1),
        0xEF => "RST $28".to_string(),

        0xF0 => format!("LDH A,($FF{:02X})", b1),
        0xF1 => "POP AF".to_string(),
        0xF2 => "LD A,($FF00+C)".to_string(),
        0xF3 => "DI".to_string(),
        0xF5 => "PUSH AF".to_string(),
        0xF6 => format!("OR ${:02X}", b1),
        0xF7 => "RST $30".to_string(),
        0xF8 => format!("LD HL,SP+${:02X}", b1),
        0xF9 => "LD SP,HL".to_string(),
        0xFA => format!("LD A,(${:04X})", nn),
        0xFB => "EI".to_string(),
        0xFE => format!("CP ${:02X}", b1),
        0xFF => "RST $38".to_string(),

        _ => format!("??? ${:02X}", op),
    }
}

fn reg_name(r: u8) -> &'static str {
    match r {
        0 => "B",
        1 => "C",
        2 => "D",
        3 => "E",
        4 => "H",
        5 => "L",
        6 => "(HL)",
        7 => "A",
        _ => "?",
    }
}

fn disassemble_cb(op: u8) -> String {
    let reg = reg_name(op & 0x7);
    match op {
        0x00..=0x07 => format!("RLC {}", reg),
        0x08..=0x0F => format!("RRC {}", reg),
        0x10..=0x17 => format!("RL {}", reg),
        0x18..=0x1F => format!("RR {}", reg),
        0x20..=0x27 => format!("SLA {}", reg),
        0x28..=0x2F => format!("SRA {}", reg),
        0x30..=0x37 => format!("SWAP {}", reg),
        0x38..=0x3F => format!("SRL {}", reg),
        0x40..=0x7F => format!("BIT {},{}", (op - 0x40) >> 3, reg),
        0x80..=0xBF => format!("RES {},{}", (op - 0x80) >> 3, reg),
        0xC0..=0xFF => format!("SET {},{}", (op - 0xC0) >> 3, reg),
    }
}
