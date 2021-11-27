extern crate clap;
#[macro_use]
extern crate log;
extern crate simplelog;

mod kakoune;
mod controller;
mod debug_adapter_comms;
mod context;
mod general;
mod stack_trace;
mod variables;
mod types;

use clap::{crate_version, Arg, App};
use std::fs::File;
use simplelog::*;

fn main() {
    //Get command line arguments
    let matches = App::new("Kak-DAP")
        .version(crate_version!())
        .arg(
            Arg::with_name("session")
                .short("s")
                .long("session")
                .value_name("SESSION")
                .help("Kakoune session to communicate with")
                .required(true)
        )
        .arg(
            Arg::with_name("log")
                .long("log")
                .value_name("PATH")
                .help("File to write the log into instead of stderr")
                .takes_value(true),
        )
        .get_matches();

    //Extract the current session
    let session : String = matches.value_of("session").map(str::to_string).unwrap();

    //Initialize the logger
    if let Some(log_path) = matches.value_of("log") {
        WriteLogger::init(
            LevelFilter::Trace,
            Config::default(),
            File::create(log_path).unwrap()).unwrap();
    }
    else {
        TermLogger::init(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Stdout,
            ColorChoice::Auto).unwrap();
    }

    //Set the dap_running option and kickstart the whole kit and kaboodle
    debug!("Starting kak-dap session");
    controller::start(&session);
}
