extern crate clap;

mod kakoune;
mod controller;
mod debug_adapter_comms;

use clap::{Arg, App};

fn main() {
    let matches = App::new("Kak-DAP")
        .version("0.1.0")
        .arg(Arg::with_name("session")
                .short("s")
                .long("session")
                .value_name("SESSION")
                .help("Kakoune session to debug in")
                .required(true))
        .get_matches();

    let session : String = matches.value_of("session").map(str::to_string).unwrap();

    kakoune::kak_command("set-option global dap_running true".to_string(), &session);

    kakoune::start_kak_comms(&session);

    controller::start(&session);
}
