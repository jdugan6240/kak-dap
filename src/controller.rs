use std::process;
use std::thread;
use std::sync::{Arc, Mutex};

use json::object;

use crate::kakoune;
use crate::debug_adapter_comms;
use crate::context::*;

pub fn start(session: &String) {
    let kakoune_rx = kakoune::start_kak_comms();
    //Begin communication with the debug adapter
    //Debug adapter hardcoded for now; TODO: make configurable
    let (adapter_tx, adapter_rx) = debug_adapter_comms::debug_start("python", &["/home/jdugan/debugpy/src/debugpy/adapter".to_string()]);

    let cxt_src = Arc::new(Mutex::new(Context::new(
        adapter_tx, session.to_string()
    )));

    {
        let ctx = cxt_src.lock().expect("Failed to lock context");
        kakoune::kak_command("set-option global dap_running true".to_string(), &ctx);
    }

    //let my_session: String = session.clone();
    //let adapter_tx = adapter_tx_src.clone();

    let ctx = Arc::clone(&cxt_src);
    thread::spawn(move || {
        for msg in adapter_rx {
            let ctx = ctx.lock().expect("Failed to lock context");
            kakoune::print_debug(&msg["type"].to_string(), &ctx);
            //TODO: parse and handle messages from the debug adapter
        }
    });
    let ctx = Arc::clone(&cxt_src);
    {
        let mut ctx = ctx.lock().expect("Failed to lock context");
        initialize(&mut ctx);
    }
    for msg in kakoune_rx {
        let ctx = ctx.lock().expect("Failed to lock context");
        parse_cmd(msg.to_string(), &ctx);
    }

}

pub fn parse_cmd(command: String, ctx: &Context) {
    //Trim the newline from the command
    let cmd = command.trim();

    //Depending on the command given, act accordingly
    if cmd == "stop" {
        kakoune::kak_command("set-option global dap_running false".to_string(), ctx);
        process::exit(0);
    }
}

pub fn initialize(ctx: &mut Context) {
    //Construct the initialize request
    let msg = object!{
        "type": "request",
        "seq": ctx.next_req_id(),
        "command": "initialize",
        "arguments": {
            "adapterID": "pydbg",
            "linesStartAt1": true,
            "columnsStartAt1": true,
            "pathFormat": "path",
        }
    };
    //Send it to the debug adapter
	ctx.debg_apt_tx.send(msg).expect("Failed to send initialize message to language server");
}
