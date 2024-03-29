use crossbeam_channel::{bounded, Receiver};
use std::io::{Read, Write};
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::UnixListener;
use std::process::{Command, Stdio};
use std::{env, fs, path, thread};

// This function sends a Kakoune command to the given Kakoune session.
pub fn kak_command(command: &str, session: &str) {
    let mut child = Command::new("kak")
        .args(&["-p", session])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let child_stdin = child.stdin.as_mut().unwrap();
    debug!("To editor: {}", command);
    child_stdin
        .write_all(command.as_bytes())
        .expect("Failed to write to stdin of child process.");
}

// Escape Kakoune string wrapped into single quote
pub fn editor_escape(s: &str) -> String {
    s.replace("'", "''")
}

// This function creates the kak-dap temp dir.
pub fn temp_dir() -> path::PathBuf {
    let mut path = env::temp_dir();
    path.push("kak-dap");
    let old_mask = unsafe { libc::umask(0) };
    // Ignoring possible error during $TMPDIR/kak-dap creation to have a chance to restore umask.
    let _ = fs::DirBuilder::new()
        .recursive(true)
        .mode(0o1777)
        .create(&path);
    unsafe {
        libc::umask(old_mask);
    }
    path
}

// This function removes the socket file.
pub fn clean_socket(session: &String) {
    let path = temp_dir();
    let sock_path = path.join(session);
    if fs::remove_file(sock_path).is_err() {
        error!("Failed to remove socket file");
    };
}

// This function spawns the thread that listens for commands on a socket
// and issues commands to the Kakoune session that spawned us.
pub fn start_kak_comms(session: &String) -> Receiver<json::JsonValue> {
    let (reader_tx, reader_rx) = bounded(1024);
    // Create socket
    let mut path = temp_dir();
    path.push(session);
    if path.exists() {
        let sock_path = path.clone();
        // Clean up dead kak-dap session
        if fs::remove_file(sock_path).is_err() {
            error!("Failed to clean up dead kak-dap session");
        }
    }
    let listener = match UnixListener::bind(&path) {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind: {}", e);
            return reader_rx;
        }
    };
    // Begin socket processing
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
                            debug!("From editor: {}", request);
                            let parsed_request = json::parse(&request).unwrap();
                            reader_tx
                                .send(parsed_request)
                                .expect("Failed to send request from Kakoune");
                        }
                        Err(e) => {
                            error!("Failed to read from stream: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    });

    reader_rx
}

