use crate::configuration::RuntimeConfig;
use crate::io::registers::IORegisters;
use crate::Byte;
use parking_lot::RwLock;
use piston_window::Key;
use std::sync::Arc;

pub struct JoypadHandler {
    io_registers: Arc<RwLock<IORegisters>>,
    runtime_config: Arc<RwLock<RuntimeConfig>>,
}

impl JoypadHandler {
    pub fn new(
        io_registers: Arc<RwLock<IORegisters>>,
        runtime_config: Arc<RwLock<RuntimeConfig>>,
    ) -> Self {
        Self {
            io_registers,
            runtime_config,
        }
    }

    pub fn press(&self, key: Key) {
        match key {
            Key::X => {
                let mut io_registers = self.io_registers.write();
                io_registers.p1.a = true;
                io_registers.interrupt_flag.set_p10_p13_transition(true);
            }
            Key::Z => {
                let mut io_registers = self.io_registers.write();
                io_registers.p1.b = true;
                io_registers.interrupt_flag.set_p10_p13_transition(true);
            }
            Key::Return => {
                let mut io_registers = self.io_registers.write();
                io_registers.p1.start = true;
                io_registers.interrupt_flag.set_p10_p13_transition(true);
            }
            Key::RShift => {
                let mut io_registers = self.io_registers.write();
                io_registers.p1.select = true;
                io_registers.interrupt_flag.set_p10_p13_transition(true);
            }
            Key::Left => {
                let mut io_registers = self.io_registers.write();
                io_registers.p1.left = true;
                io_registers.interrupt_flag.set_p10_p13_transition(true);
            }
            Key::Right => {
                let mut io_registers = self.io_registers.write();
                io_registers.p1.right = true;
                io_registers.interrupt_flag.set_p10_p13_transition(true);
            }
            Key::Up => {
                let mut io_registers = self.io_registers.write();
                io_registers.p1.up = true;
                io_registers.interrupt_flag.set_p10_p13_transition(true);
            }
            Key::Down => {
                let mut io_registers = self.io_registers.write();
                io_registers.p1.down = true;
                io_registers.interrupt_flag.set_p10_p13_transition(true);
            }
            Key::M => {
                self.runtime_config.write().toggle_mute();
            }
            Key::Space => {
                self.runtime_config.write().user_speed_multiplier = 20;
            }
            Key::R => {
                self.runtime_config.write().set_reset(true);
            }
            Key::D => {
                self.runtime_config.write().toggle_debug();
            }
            _ => {}
        };
    }

    pub fn release(&self, key: Key) {
        let mut io_registers = self.io_registers.write();
        match key {
            Key::X => io_registers.p1.a = false,
            Key::Z => io_registers.p1.b = false,
            Key::Return => io_registers.p1.start = false,
            Key::RShift => io_registers.p1.select = false,
            Key::Left => io_registers.p1.left = false,
            Key::Right => io_registers.p1.right = false,
            Key::Up => io_registers.p1.up = false,
            Key::Down => io_registers.p1.down = false,
            Key::Space => self.runtime_config.write().user_speed_multiplier = 1,
            _ => {}
        }
    }
}

#[derive(Default)]
pub struct Joypad {
    // P14 - P10
    pub right: bool,
    // P14 - P11
    pub left: bool,
    // P14 - P12
    pub up: bool,
    // P14 - P13
    pub down: bool,

    // P15 - P10
    pub a: bool,
    // P15 - P11
    pub b: bool,
    // P15 - P12
    pub select: bool,
    // P15 - P13
    pub start: bool,

    p14: bool,
    p15: bool,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            right: false,
            left: false,
            up: false,
            down: false,

            a: false,
            b: false,
            select: false,
            start: false,

            p14: false,
            p15: false,
        }
    }

    pub fn to_byte(&self) -> Byte {
        if !self.p15 && !self.p14 {
            return 0xFF;
        }

        let mut value = (!self.p15 as Byte) << 5;
        value |= (!self.p14 as Byte) << 4;

        if self.p15 {
            value |= (!(self.start) as Byte) << 3;
            value |= (!(self.select) as Byte) << 2;
            value |= (!(self.b) as Byte) << 1;
            value |= !(self.a) as Byte;
        } else if self.p14 {
            value |= (!(self.down) as Byte) << 3;
            value |= (!(self.up) as Byte) << 2;
            value |= (!(self.left) as Byte) << 1;
            value |= !(self.right) as Byte;
        }

        value
    }

    pub fn parse_byte(&mut self, new_value: Byte) {
        self.p14 = new_value & 0b10000 != 0b10000;
        self.p15 = new_value & 0b100000 != 0b100000;
    }
}
