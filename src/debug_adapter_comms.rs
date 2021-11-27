use crossbeam_channel::{bounded, Receiver, Sender};
//use fnv::FnvHashMap;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, BufWriter, Error, ErrorKind, Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use json::object;

use crate::context::*;

//Start the debug adapter process and connect to its stdio.
pub fn debug_start(cmd: &str, args: &[String]) -> (Sender<json::JsonValue>, Receiver<json::JsonValue>) {
    //Spawn debug adapter process and capture stdio
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start debug adapter");

    //Obtain reader and writer objects for the child process
    let writer = BufWriter::new(child.stdin.take().expect("Failed to open stdin"));
    let reader = BufReader::new(child.stdout.take().expect("Failed to open stdout"));

    //Temporary way of tracing debug adapter errors
    //Print any errors to the Kakoune debug buffer
    let mut stderr = BufReader::new(child.stderr.take().expect("Failed to open stderr"));
    //thread::spawn(move || loop {
    thread::spawn(move || loop {
        let mut buf = String::new();
        stderr.read_to_string(&mut buf).unwrap();
        if buf.is_empty() {
            continue;
        }
        error!("Debug adapter error: {}", buf);
    });

    let (reader_tx, reader_rx) = bounded(1024);
    thread::spawn(move || {
        reader_loop(reader, &reader_tx).expect("Failed to read message from debug adapter");
    });

    let (writer_tx, writer_rx): (Sender<json::JsonValue>, Receiver<json::JsonValue>) = bounded(1024);
    thread::spawn(move || {
        writer_loop(writer, &writer_rx).expect("Failed to write message to debug adapter");
    });

    (writer_tx, reader_rx)
}

//Thread to read the stdout of the debug adapter process.
fn reader_loop(mut reader: impl BufRead, tx: &Sender<json::JsonValue>) -> io::Result<()> {
    //Store headers of message being received
    //Used to determine if Content-Length header has been received
    let mut headers = HashMap::new();
    loop {
        headers.clear();
        loop {
            let mut header = String::new();
            if reader.read_line(&mut header)? == 0 {
                return Err(Error::new(ErrorKind::Other, "Failed to read from adapter"));
            }
            let header = header.trim();
            if header.is_empty() {
                break;
            }
            let parts: Vec<&str> = header.split(": ").collect();
            if parts.len() != 2 {
                return Err(Error::new(ErrorKind::Other, "Failed to parse header"));
            }
            headers.insert(parts[0].to_string(), parts[1].to_string());
        }
        //Get the length of the message we are receiving
        let content_len = headers
            .get("Content-Length")
            .expect("Failed to find Content-Length header")
            .parse()
            .expect("Failed to parse Content-Length header");
        //Now read that many characters to obtain the message
        let mut content = vec![0; content_len];
        reader.read_exact(&mut content)?;
        let msg = String::from_utf8(content).expect("Failed to read content as UTF-8 string");
        let output = json::parse(&msg.to_string()).unwrap();
        let output_cln = output.clone();
        debug!("From debug adapter: {}", json::stringify_pretty(output_cln, 4));
        if output.is_object() {
            tx.send(output).expect("Failed to send message from debug adapter");
        }
    }
}

//Thread to write to the stdin of the debug adapter process.
fn writer_loop(mut writer: impl Write, rx: &Receiver<json::JsonValue>) -> io::Result<()> {
    for request in rx {
        let request = request.dump();
        write!(
            writer,
            "Content-Length: {}\r\n\r\n{}",
            request.len(),
            request
        )?;
        writer.flush()?;
    }
    Ok(())
}

//Sends a request to the debug adapter.
pub fn do_request(cmd: String, args: json::JsonValue, ctx: &mut Context) {
    let msg = object!{
        "type": "request",
        "seq": ctx.next_req_id(),
        "command": cmd,
        "arguments": args
    };

    let msg_cln = msg.clone();
    let msg_pretty = msg_cln.clone();

    debug!("To debug adapter: {}", json::stringify_pretty(msg_pretty, 4));

    //Send it to the debug adapter
    ctx.debg_apt_tx.send(msg).expect("Failed to send message to debug adapter");

    //Add it to the pending requests list
    ctx.cur_requests.push(msg_cln);
}

//Sends a response to the debug adapter.
//Currently, only one response is sent by the client: the response to the runInTerminal command.
pub fn do_response(cmd: String, body: json::JsonValue, ctx: &mut Context) {
    let msg = object!{
        "type": "response",
        "seq": ctx.next_req_id(),
        "request_seq": ctx.last_adapter_seq,
        "command": cmd,
        "success": true,
        "body": body
    };
    let msg_cln = msg.clone();
    debug!("To debug adapter: {}", json::stringify_pretty(msg_cln, 4));
    //Send it to the debug adapter
    ctx.debg_apt_tx.send(msg).expect("Failed to send response to debug adapter");
}
