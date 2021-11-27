use std::io::{Read, Write};
use std::process::{Command, Stdio};
use crossbeam_channel::{bounded, Receiver};
use std::{env, fs, path, thread};
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::UnixListener;

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

/// Escape Kakoune string wrapped into single quote
pub fn editor_escape(s: &str) -> String {
    s.replace("'", "''")
}

//This function creates the kak-dap temp dir.
pub fn temp_dir() -> path::PathBuf {
    let mut path = env::temp_dir();
    path.push("kak-dap");
    let old_mask = unsafe { libc::umask(0) };
    //Ignoring possible error during $TMPDIR/kak-dap creation to have a chance to restore umask.
    let _ = fs::DirBuilder::new()
        .recursive(true)
        .mode(0o1777)
        .create(&path);
    unsafe {
        libc::umask(old_mask);
    }
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&path)
        .unwrap();
    path
}

//This function removes the socket file.
pub fn clean_socket(session: &String) {
    let path = temp_dir();
    let sock_path = path.join(session);
    if fs::remove_file(sock_path).is_err() {
        println!("Failed to remove socket file");
    };
}

//This function spawns the thread that listens for commands on a socket
//and issues commands to the Kakoune session that spawned us.
pub fn start_kak_comms(session: &String) -> Receiver<String> {
    let (reader_tx, reader_rx) = bounded(1024);
    //Create socket
    let mut path = temp_dir();
    path.push(session);
    if path.exists() {
        let sock_path = path.clone();
        //Clean up dead kak-dap session
        if fs::remove_file(sock_path).is_err() {
            println!("Failed to clean up dead kak-dap session");
        }
    }
    let listener = match UnixListener::bind(&path) {
        Ok(listener) => listener,
        Err(e) => {
            println!("Failed to bind: {}", e);
            return reader_rx;
        }
    };
    //Begin socket processing
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut request = String::new();
                    match stream.read_to_string(&mut request) {
                        Ok(_) => {
                            if request.is_empty() {
                                continue;
                            }
                            println!("From editor: {}", request);
                            reader_tx.send(request).expect("Failed to send request from Kakoune");
                        }
                        Err(e) => {
                            println!("Failed to read from stream: {}", e);
                        }
                    }
                }
                Err (e) => {
                    println!("Failed to accept connection: {}", e);
                }
            }
        }
    });

    reader_rx
}

