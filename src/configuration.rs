use crate::cpu::Cpu;
use clap::{Arg, Command};

#[readonly::make]
pub struct Configuration {
    pub debug_header: bool,
    pub bootstrap: bool,
    pub rom_file: String,

    pub user_speed_multiplier: i32,
}

impl Configuration {
    pub fn from_command(app_name: &'static str) -> Self {
        let matches = Command::new(app_name)
            .arg(
                Arg::new("ROMFILE")
                    .required(true)
                    .index(1)
                    .help("Path of the ROM file to use"),
            )
            .arg(
                Arg::new("debug-header")
                    .long("debug-header")
                    .help("Prints the parsed cartridge header"),
            )
            .arg(
                Arg::new("bootstrap")
                    .long("bootstrap")
                    .help("Uses bootstrap ROM"),
            )
            .get_matches();

        Self {
            debug_header: matches.contains_id("debug-header"),
            bootstrap: matches.contains_id("bootstrap"),
            rom_file: matches.get_one::<String>("ROMFILE").unwrap().to_string(),

            user_speed_multiplier: 1,
        }
    }
}

pub struct RuntimeConfig {
    pub user_speed_multiplier: i32,
    pub muted: bool,
    pub available_cycles: i32,
    pub reset: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            user_speed_multiplier: 1,
            muted: false,
            available_cycles: Cpu::AVAILABLE_CCYCLES_PER_FRAME,
            reset: false,
        }
    }
}

impl RuntimeConfig {
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    pub fn has_been_reset(&self) -> bool {
        self.reset
    }

    pub fn set_reset(&mut self, value: bool) {
        self.reset = value;
    }

    pub fn reset_available_ccycles(&mut self) {
        self.available_cycles = Cpu::AVAILABLE_CCYCLES_PER_FRAME * self.user_speed_multiplier;
    }

    pub fn cpu_has_available_ccycles(&self) -> bool {
        self.available_cycles > 0
    }
}
