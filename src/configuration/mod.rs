use crate::cpu::cpu::CPU;
use clap::{App, Arg};

#[readonly::make]
pub struct Configuration {
    pub debug_audio: bool,
    pub debug_header: bool,
    pub bootstrap: bool,
    pub rom_file: String,

    pub user_speed_multiplier: i32,
}

impl Configuration {
    pub fn from_command(app_name: &str) -> Self {
        let matches = App::new(app_name)
            .arg(
                Arg::with_name("ROMFILE")
                    .required(true)
                    .index(1)
                    .help("Path of the ROM file to use"),
            )
            .arg(
                Arg::with_name("debug-cpu")
                    .long("debug-cpu")
                    .help("Prints CPU instructions on command line"),
            )
            .arg(
                Arg::with_name("debug-audio")
                    .long("debug-audio")
                    .help("Prints audio triggered on command line"),
            )
            .arg(
                Arg::with_name("debug-header")
                    .long("debug-header")
                    .help("Prints the parsed cartridge header"),
            )
            .arg(
                Arg::with_name("bootstrap")
                    .long("bootstrap")
                    .help("Uses bootstrap ROM"),
            )
            .get_matches();

        Self {
            debug_audio: matches.is_present("debug-audio"),
            debug_header: matches.is_present("debug-header"),
            bootstrap: matches.is_present("bootstrap"),
            rom_file: matches.value_of("ROMFILE").unwrap().to_string(),

            user_speed_multiplier: 1,
        }
    }
}

pub struct RuntimeConfig {
    pub user_speed_multiplier: i32,
    pub muted: bool,
    pub available_cycles: i32,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            user_speed_multiplier: 1,
            muted: false,
            available_cycles: CPU::AVAILABLE_CCYCLES_PER_FRAME,
        }
    }
}

impl RuntimeConfig {
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    pub fn reset_available_ccycles(&mut self) {
        self.available_cycles = CPU::AVAILABLE_CCYCLES_PER_FRAME * self.user_speed_multiplier;
    }

    pub fn cpu_has_available_ccycles(&self) -> bool {
        return self.available_cycles > 0;
    }
}
