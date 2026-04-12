use crate::cpu::Cpu;
use clap::{Arg, Command};

#[readonly::make]
pub struct Configuration {
    pub debug_header: bool,
    pub bootstrap_path: Option<String>,
    pub rom_file: String,

    pub user_speed_multiplier: i32,
    pub trace: bool,
}

impl Configuration {
    pub fn from_command(app_name: &'static str) -> Self {
        let command = Command::new(app_name)
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
            );

        #[cfg(debug_assertions)]
        let command = command.arg(
            Arg::new("trace")
                .long("trace")
                .num_args(0)
                .help("Print each instruction as it executes (dev builds only)"),
        );

        let matches = command.get_matches();

        #[cfg(debug_assertions)]
        let trace = matches.contains_id("trace");
        #[cfg(not(debug_assertions))]
        let trace = false;

        Self {
            debug_header: matches.contains_id("debug-header"),
            bootstrap_path: matches
                .get_one::<String>("bootstrap")
                .map(|x| x.to_string()),
            rom_file: matches.get_one::<String>("ROMFILE").unwrap().to_string(),

            user_speed_multiplier: 1,
            trace,
        }
    }
}

pub struct RuntimeConfig {
    pub user_speed_multiplier: i32,
    pub muted: bool,
    pub available_cycles: i32,
    pub reset: bool,
    pub debug: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            user_speed_multiplier: 1,
            muted: false,
            available_cycles: Cpu::AVAILABLE_CCYCLES_PER_FRAME,
            reset: false,
            debug: false,
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

    pub fn is_debug(&self) -> bool {
        self.debug
    }

    pub fn toggle_debug(&mut self) {
        self.debug = !self.debug;
    }
}
