use crate::bus::address::Address;
use crate::Word;
use std::fmt::{Display, Formatter};

// CPU
pub const CPU_PC_WATCHPOINTS: [Word; 0] = [];

// I/O
pub const IO_READ_WATCHPOINTS: [Word; 0] = [];
pub const IO_WRITE_WATCHPOINTS: [Word; 0] = [];

pub trait Debuggable {
    fn output_debug(&self);
}

pub struct OutputDebug {}

impl OutputDebug {
    pub fn print_reason(debug_reason: DebugReason) {
        println!(
            "#### {} ########################################################################\n",
            debug_reason
        )
    }

    pub fn print_before() {
        println!(
            "---- BEFORE ---------------------------------------------------------------------",
        );
    }

    pub fn print_reason_with_before(debug_reason: DebugReason) {
        Self::print_reason(debug_reason);
        Self::print_before();
    }

    pub fn print_after() {
        println!(
            "---- AFTER ----------------------------------------------------------------------",
        );
    }
}

pub enum DebugReason {
    IORead(Word),
    IOWrite(Word),
    PC(Word),
}

impl Display for DebugReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            DebugReason::IORead(addr) => format!("I/O Read {:X}", addr),
            DebugReason::IOWrite(addr) => format!("I/O Write {:X}", addr),
            DebugReason::PC(addr) => format!("PC {:X}", addr),
        };

        write!(f, "{}", text)
    }
}
