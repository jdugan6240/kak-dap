extern crate clap;

mod kakoune;
mod controller;
mod debug_adapter_comms;
mod context;
mod general;
mod stack_trace;
mod variables;
mod types;

use clap::{Arg, App};
use std::env;

fn main() {
    //Get command line arguments
    let _matches = App::new("Kak-DAP")
        .version("0.1.0")
        .get_matches();

    //Extract the current session
    let session;
    match env::var("kakoune_session") {
        Ok(val) => session = val,
        Err(_e) => session = "none".to_string(),
    }
    println!("{}", session);
    //If we have a valid session, set the dap_running option and kickstart the whole kit and kaboodle
    if session != "none" {
        controller::start(&session);
    }
}
