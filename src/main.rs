extern crate clap;
#[macro_use]
extern crate log;
extern crate simplelog;

mod breakpoints;
mod config;
mod context;
mod controller;
mod debug_adapter_comms;
mod general;
mod kakoune;
mod stack_trace;
mod types;
mod variables;

use clap::{crate_version, App, Arg};
use simplelog::*;
use std::fs;
use std::fs::File;
use std::panic;

use std::io::{stdin, Read, Write};
use std::os::unix::net::UnixStream;

use json::object;

fn main() {
    // Get command line arguments
    let matches = App::new("Kak-DAP")
        .version(crate_version!())
        .arg(
            Arg::with_name("session")
                .short("s")
                .long("session")
                .value_name("SESSION")
                .help("Kakoune session to communicate with")
                .required(true),
        )
        .arg(
            Arg::with_name("log")
                .long("log")
                .value_name("PATH")
                .help("File to write the log into instead of stderr")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("request")
                .long("request")
                .help("Forward stdin to kak-dap server")
        )
        .get_matches();

    // Enable logging of panics
    panic::set_hook(Box::new(|panic_info| {
        error!("panic: {}", panic_info);
    }));

    // Extract the current session
    let session: String = matches.value_of("session").map(str::to_string).unwrap();

    // Initialize the logger
    if let Some(log_path) = matches.value_of("log") {
        WriteLogger::init(
            LevelFilter::Trace,
            Config::default(),
            File::create(log_path).unwrap(),
        )
        .unwrap();
    } else {
        TermLogger::init(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Stdout,
            ColorChoice::Auto,
        )
        .unwrap();
    }
    if matches.is_present("request") {
        // Forward the stdin to the kak-dap server
        let mut input = Vec::new();
        stdin()
            .read_to_end(&mut input)
            .expect("Failed to read stdin");
        let mut path = kakoune::temp_dir();
        path.push(session);
        if let Ok(mut stream) = UnixStream::connect(&path) {
            stream
                .write_all(&input)
                .expect("Failed to send stdin to server");
        }
    } else {
        // If we are receiving breakpoints from the breakpoints file, get them
        let mut breakpoints = object! {};
        let mut path = kakoune::temp_dir();
        path.push(format!("{}_breakpoints", session));
        debug!(
            "Searching for breakpoints on path {}",
            path.to_string_lossy()
        );
        if path.exists() {
            let break_path = path.clone();
            let contents = fs::read_to_string(path).expect("Couldn't read from file");
            breakpoints = json::parse(&contents).expect("Couldn't convert contents to JSON");
            debug!("Breakpoint data: {}", breakpoints.to_string());
            if fs::remove_file(break_path).is_err() {
                error!("Couldn't clean up breakpoints file");
            }
        }

        // Set the dap_running option and kickstart the whole kit and kaboodle
        debug!("Starting kak-dap session");
        controller::start(&session, breakpoints);
    }
}
