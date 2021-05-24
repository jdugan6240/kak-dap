use std::io;
use std::io::{Write};
use std::process::{Command, Stdio};
use crossbeam_channel::{bounded, Receiver};
use std::thread;

use crate::context::*;

//This function sends a Kakoune command to the given Kakoune session.
pub fn kak_command(command: String, ctx: &Context) {
    let mut child = Command::new("kak")
        .args(&["-p", &ctx.session])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stdout(Stdio::null())
        .spawn().unwrap();
    let child_stdin = child.stdin.as_mut().unwrap();
    child_stdin.write_all(command.as_bytes()).expect("Failed to write to stdin of child process.");
}

//This function prints to the Kakoune debug buffer.
pub fn print_debug(message: &String, ctx: &Context) {
    let cmd = format!("echo -debug '{}'", message);
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

