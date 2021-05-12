use std::process;
use std::thread;
use crossbeam_channel::{Sender};
//use crossbeam_utils::thread;

use json::object;

use crate::kakoune;
use crate::debug_adapter_comms;

pub fn start(session: &String) {
    let kakoune_rx = kakoune::start_kak_comms();
    //Begin communication with the debug adapter
    //Debug adapter hardcoded for now; TODO: make configurable
    let (adapter_tx_src, adapter_rx) = debug_adapter_comms::debug_start("python", &["/home/jdugan/debugpy/src/debugpy/adapter".to_string()]);

    let my_session: String = session.clone();

    thread::spawn(move || {
        for msg in adapter_rx {
            //TODO: parse and handle messages from the debug adapter
            //println!("{}", msg.to_string());
            kakoune::print_debug(&msg.to_string(), &my_session);
        }
    });
    let adapter_tx = adapter_tx_src.clone();
    let session_cln = session.clone();
    initialize(adapter_tx);
    for msg in kakoune_rx {
        parse_cmd(msg.to_string(), &session_cln);
    }

}

pub fn parse_cmd(command: String, session: &String) {
    //Trim the newline from the command
    let cmd = command.trim();

    //Depending on the command given, act accordingly
    if cmd == "stop" {
        kakoune::kak_command("set-option global dap_running false".to_string(), session);
        process::exit(0);
    }
}

pub fn initialize(adapter_tx: Sender<json::JsonValue>) {
    //Construct the initialize request
    let msg = object!{
        "type": "request",
        "seq": 1,
        "command": "initialize",
        "arguments": {
            "adapterID": "pydbg",
            "linesStartAt1": true,
            "columnsStartAt1": true,
            "pathFormat": "path",
        }
    };
    //Send it to the debug adapter
	adapter_tx.send(msg).expect("Failed to send initialize message to language server");
}
