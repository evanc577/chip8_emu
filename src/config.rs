use crate::drivers::KeyMapping;
use clap::{value_t, App, Arg};
use std::ffi::OsString;

pub struct Config {
    pub rom_file: OsString,
    pub rate: Option<u64>,
    pub key_map: KeyMapping,
}

pub fn get_config() -> Config {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("rom")
                .value_name("FILE")
                .required(true)
                .help("Path to ROM file"),
        )
        .arg(
            Arg::with_name("rate")
                .short("r")
                .long("rate")
                .value_name("NUM")
                .default_value("700")
                .help("Instructions executed per second. '0' for no limit"),
        )
        .arg(
            Arg::with_name("key_map")
                .short("k")
                .long("key-map")
                .possible_values(&KeyMapping::variants())
                .case_insensitive(true)
                .default_value("QWERTY")
                .help("Keyboard mapping"),
        )
        .get_matches();

    let rom_file = matches.value_of_os("rom").unwrap().to_owned();
    let rate = match value_t!(matches.value_of("rate"), u64).unwrap_or_else(|e| e.exit()) {
        0 => None,
        i => Some(i),
    };
    let key_map = value_t!(matches, "key_map", KeyMapping).unwrap_or_else(|e| e.exit());

    Config {
        rom_file,
        rate,
        key_map,
    }
}
