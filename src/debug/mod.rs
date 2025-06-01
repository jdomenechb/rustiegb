use crate::bus::address::Address;
use crate::{Byte, Word};
use prettytable::{cell, row, Table};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

// CPU
pub const CPU_PC_WATCHPOINTS: [Word; 0] = [];

// I/O
pub const IO_READ_WATCHPOINTS: [Word; 0] = [];
pub const IO_WRITE_WATCHPOINTS: [Word; 0] = [];

pub trait Debuggable {
    fn get_debug_values(&self) -> BTreeMap<&str, String>;
}

pub struct OutputDebug {
    debug_reason: DebugReason,
    situations: Vec<String>,
    values: BTreeMap<String, BTreeMap<String, String>>,
}

impl OutputDebug {
    pub fn new_with_reason(debug_reason: DebugReason) -> Self {
        Self {
            debug_reason,
            situations: vec![],
            values: BTreeMap::new(),
        }
    }

    pub fn push_situation(&mut self, situation: &str, values: BTreeMap<&str, String>) {
        self.situations.push(situation.to_string());

        for (item_header, value) in values {
            self.values
                .entry(item_header.to_string())
                .and_modify(|item| {
                    item.insert(situation.to_string(), value.clone());
                })
                .or_insert(BTreeMap::from([(situation.to_string(), value)]));
        }
    }

    pub fn print(&self) {
        println!(
            "#### {} ########################################################################\n",
            self.debug_reason
        );

        let mut table = Table::new();
        let mut headers_row = row!["Register / Address"];

        for situation in &self.situations {
            headers_row.add_cell(cell!(situation))
        }

        table.add_row(headers_row);

        for (register_or_address_name, situation_values) in &self.values {
            let mut row = row![register_or_address_name];

            for situation in &self.situations {
                let value = situation_values
                    .get(situation)
                    .map_or("".to_string(), |x| x.to_string());
                row.add_cell(cell!(value))
            }

            table.add_row(row);
        }

        table.printstd();
        println!();
    }
}

pub enum DebugReason {
    IORead(Word),
    IOWrite(Word, Byte),
    PC(Word),
}

impl Display for DebugReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            DebugReason::IORead(addr) => format!("I/O Read {:X}", addr),
            DebugReason::IOWrite(addr, value) => format!("I/O Write {:X} value {:X}", addr, value),
            DebugReason::PC(addr) => format!("PC {:X}", addr),
        };

        write!(f, "{}", text)
    }
}
