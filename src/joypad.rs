use crate::configuration::RuntimeConfig;
use crate::memory::Memory;
use parking_lot::RwLock;
use piston_window::Key;
use std::sync::Arc;

pub struct JoypadHandler {
    memory: Arc<RwLock<Memory>>,
    runtime_config: Arc<RwLock<RuntimeConfig>>,
}

impl JoypadHandler {
    pub fn new(memory: Arc<RwLock<Memory>>, runtime_config: Arc<RwLock<RuntimeConfig>>) -> Self {
        Self {
            memory,
            runtime_config,
        }
    }

    pub fn press(&self, key: Key) {
        match key {
            Key::X => {
                let mut memory = self.memory.write();
                memory.joypad().a = true;
                memory.interrupt_flag().set_p10_p13_transition(true);
            }
            Key::Z => {
                let mut memory = self.memory.write();
                memory.joypad().b = true;
                memory.interrupt_flag().set_p10_p13_transition(true);
            }
            Key::Return => {
                let mut memory = self.memory.write();
                memory.joypad().start = true;
                memory.interrupt_flag().set_p10_p13_transition(true);
            }
            Key::RShift => {
                let mut memory = self.memory.write();
                memory.joypad().select = true;
                memory.interrupt_flag().set_p10_p13_transition(true);
            }
            Key::Left => {
                let mut memory = self.memory.write();
                memory.joypad().left = true;
                memory.interrupt_flag().set_p10_p13_transition(true);
            }
            Key::Right => {
                let mut memory = self.memory.write();
                memory.joypad().right = true;
                memory.interrupt_flag().set_p10_p13_transition(true);
            }
            Key::Up => {
                let mut memory = self.memory.write();
                memory.joypad().up = true;
                memory.interrupt_flag().set_p10_p13_transition(true);
            }
            Key::Down => {
                let mut memory = self.memory.write();
                memory.joypad().down = true;
                memory.interrupt_flag().set_p10_p13_transition(true);
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
            _ => {}
        };
    }

    pub fn release(&self, key: Key) {
        match key {
            Key::X => self.memory.write().joypad().a = false,
            Key::Z => self.memory.write().joypad().b = false,
            Key::Return => self.memory.write().joypad().start = false,
            Key::RShift => self.memory.write().joypad().select = false,
            Key::Left => self.memory.write().joypad().left = false,
            Key::Right => self.memory.write().joypad().right = false,
            Key::Up => self.memory.write().joypad().up = false,
            Key::Down => self.memory.write().joypad().down = false,
            Key::Space => self.runtime_config.write().user_speed_multiplier = 1,
            _ => {}
        }
    }
}
