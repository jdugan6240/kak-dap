extern crate clap;

mod kakoune;
mod controller;
mod debug_adapter_comms;
mod context;
mod general;
mod stack_trace;
mod variables;
mod types;

use clap::{crate_version, Arg, App};

fn main() {
    //Get command line arguments
    let matches = App::new("Kak-DAP")
        .version(crate_version!())
        .arg(Arg::with_name("session")
                .short("s")
                .long("session")
                .value_name("SESSION")
                .help("Kakoune session to communicate with")
                .required(true))
        .get_matches();

    //Extract the current session
    let session : String = matches.value_of("session").map(str::to_string).unwrap();

    //Set the dap_running option and kickstart the whole kit and kaboodle
    controller::start(&session);
}
