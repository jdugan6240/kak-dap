use std::io;
use std::process::Command;
use crossbeam_channel::{bounded, Receiver};
//use crossbeam_utils::thread;
use std::thread;

//use crate::controller;
use crate::context::*;

//This function sends a Kakoune command to the given Kakoune session.
pub fn kak_command(command: String, ctx: &Context) {
    let cmd = format!("echo {} | kak -p {}", command, ctx.session);
    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("Couldn't execute command");
}

//This function prints to the Kakoune debug buffer.
pub fn print_debug(message: &String, ctx: &Context) {
    let cmd = format!("echo -debug {}", message);
    kak_command(cmd, ctx);
}

//This function spawns the thread that listens for commands on STDIO
//and issues commands to the Kakoune session that spawned us.
pub fn start_kak_comms() -> Receiver<String> {
    let (reader_tx, reader_rx) = bounded(1024);
    //Begin stdin processing
    thread::spawn(move || {
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to get input");
            reader_tx.send(input).expect("Failed to send request from Kakoune");
        }
    });

    reader_rx
}

