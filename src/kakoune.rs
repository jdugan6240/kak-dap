use std::io;
use std::process::Command;

use crate::controller;

//This function sends a Kakoune command to the given Kakoune session.
pub fn kak_command(command: String, session: &String) {
    let cmd = format!("echo {} | kak -p {}", command, session);
    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("Couldn't execute command");
}

//This function prints to the Kakoune debug buffer.
pub fn print_debug(message: String, session: &String) {
    let cmd = format!("echo -debug {}", message);
    kak_command(cmd, session);
}

//This function spawns the thread that listens for commands on STDIO
//and issues commands to the Kakoune session that spawned us.
pub fn start_kak_comms(session: &String) {
    let mut input = String::new();
    //Begin stdin processing
    //TODO: determine if this should be separate thread.
    loop {
        io::stdin().read_line(&mut input).expect("Failed to get input");
        controller::parse_cmd(&input, &session);
    };
}


