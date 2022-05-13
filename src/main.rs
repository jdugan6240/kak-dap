extern crate clap;
extern crate slog;
#[macro_use]
extern crate slog_scope;

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
use sloggers::file::FileLoggerBuilder;
use sloggers::terminal::{Destination, TerminalLoggerBuilder};
use sloggers::types::Severity;
use sloggers::Build;
use std::fs;
use std::panic;
use std::env;

use std::io::{stdin, Read, Write};
use std::os::unix::net::UnixStream;

use itertools::Itertools;
use json::object;

fn setup_logger(matches: &clap::ArgMatches<'_>) -> slog_scope::GlobalLoggerGuard {
    let mut verbosity = matches.occurrences_of("v") as u8;

    if verbosity == 0 {
        verbosity = 2
    }

    let level = match verbosity {
        0 => Severity::Error,
        1 => Severity::Warning,
        2 => Severity::Info,
        3 => Severity::Debug,
        _ => Severity::Trace,
    };

    let logger = if let Some(log_path) = matches.value_of("log") {
        // First remove the existing logfile. We don't want multiple logging sessions in a single file.
        let _result = fs::remove_file(log_path);

        let mut builder = FileLoggerBuilder::new(log_path);
        builder.level(level);
        builder.build().unwrap()
    } else {
        let mut builder = TerminalLoggerBuilder::new();
        builder.level(level);
        builder.destination(Destination::Stderr);
        builder.build().unwrap()
    };

    panic::set_hook(Box::new(|panic_info| {
        error!("panic: {}", panic_info);
    }));

    slog_scope::set_global_logger(logger)
}

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
        .arg(
            Arg::with_name("kakoune")
                .long("kakoune")
                .help("Generate commands for Kakoune to plug in kak-dap")
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity (use up to 4 times)"),
        )
        .get_matches();

    // Enable logging of panics
    panic::set_hook(Box::new(|panic_info| {
        error!("panic: {}", panic_info);
    }));

    // Extract the current session
    let session: String = matches.value_of("session").map(str::to_string).unwrap();

    // Initialize the logger
    let _guard = setup_logger(&matches);

    if matches.is_present("kakoune") {
        // Grab ../rc/kak-dap.kak and print it out
        let script: &str = include_str!("../rc/kak-dap.kak");
        let args = env::args()
            .skip(1)
            .filter(|arg| arg != "--kakoune")
            .join(" ");
        let cmd = env::current_exe().unwrap();
        let cmd = cmd.to_str().unwrap();
        let lsp_cmd = format!(
            "set global lsp_cmd '{} {}'",
            kakoune::editor_escape(cmd),
            kakoune::editor_escape(&args)
        );
        println!("{}\n{}", script, lsp_cmd);
    } else if matches.is_present("request") {
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
