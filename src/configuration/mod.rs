use clap::{App, Arg};

#[readonly::make]
pub struct Configuration {
    pub debug_cpu: bool,
    pub debug_audio: bool,
    pub debug_header: bool,
    pub bootstrap: bool,
    pub rom_file: String,
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
            debug_cpu: matches.is_present("debug-cpu"),
            debug_audio: matches.is_present("debug-audio"),
            debug_header: matches.is_present("debug-header"),
            bootstrap: matches.is_present("bootstrap"),
            rom_file: matches.value_of("ROMFILE").unwrap().to_string(),
        }
    }
}
