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

    //Set dap_running option in Kakoune session
    {
        let ctx = cxt_src.lock().expect("Failed to lock context");
        kakoune::kak_command("set-option global dap_running true".to_string(), &ctx);
    }

    //Debug adapter response handling thread
    let ctx = Arc::clone(&cxt_src);
    thread::spawn(move || {
        for msg in adapter_rx {
            let mut ctx = ctx.lock().expect("Failed to lock context");
            kakoune::print_debug(&msg.to_string(), &ctx);
            println!("{}", &msg.to_string());
            //TODO: parse and handle messages from the debug adapter
            if msg["type"].to_string() == "response" {
                handle_adapter_response(msg, &mut ctx);
            }
            else if msg["type"].to_string() == "event" {
                handle_adapter_event(msg, &mut ctx);
            }
            else if msg["type"].to_string() == "request" {
                handle_run_in_terminal_request(msg, &mut ctx);
            }
        }
    });
    //Initialize the debug adapter
    let ctx = Arc::clone(&cxt_src);
    {
        let mut ctx = ctx.lock().expect("Failed to lock context");
        initialize(&mut ctx);
    }
    //Handle messages from Kakoune
    for msg in kakoune_rx {
        let mut ctx = ctx.lock().expect("Failed to lock context");
        parse_cmd(msg.to_string(), &mut ctx);
    }

}

//Handle events from the debug adapter.
pub fn handle_adapter_event(msg: json::JsonValue, ctx: &mut Context) {
    match msg["event"].to_string().as_str() {
        "initialized" => handle_initialized_event(msg, ctx),
        _ => (),
    };
}

//Handle responses from the debug adapter.
pub fn handle_adapter_response(msg: json::JsonValue, ctx: &mut Context) {
    match msg["command"].to_string().as_str() {
        "initialize" => handle_initialize_response(msg, ctx),
        "setBreakpoints" => handle_set_breakpoint_response(msg, ctx),
        _ => (),
    };
}

//Handle commands from Kakoune.
pub fn parse_cmd(command: String, ctx: &mut Context) {
    //Trim the newline from the command
    let cmd = command.trim();

    //Depending on the command given, act accordingly
    if cmd == "stop" {
        kakoune::kak_command("set-option global dap_running false".to_string(), ctx);
        process::exit(0);
    }
    else if cmd == "continue" {
        //Send a continue command to the debugger
        let msg = object!{
            "type": "request",
            "seq": ctx.next_req_id(),
            "command": "continue",
            "arguments": {
                "threadId": 1
            }
        };
        //Send it to the debug adapter
        ctx.debg_apt_tx.send(msg).expect("Failed to send continue message to debug adapter");
    }
    else if cmd.starts_with("pid") {
        kakoune::print_debug(&"PID received".to_string(), ctx);
        //let split = cmd.split(" ");
        //let mut args = split.collect::<Vec<&str>>();
        //let pid = args.pop().unwrap().parse::<u64>().unwrap();
        let msg = object!{
            "type": "response",
            "seq": ctx.next_req_id(),
            "request_seq": ctx.last_adapter_seq,
            "command": "runInTerminal",
            "success": true,
            //"body": {
            //    "processId": pid,
            //}
        };
        kakoune::print_debug(&ctx.last_adapter_seq.to_string(), ctx);
        //println!("{}", pid);
        //Send it to the debug adapter
        ctx.debg_apt_tx.send(msg).expect("Failed to send response to debug adapter");
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
            "supportsRunInTerminalRequest": true,
        }
    };
    //Send it to the debug adapter
    ctx.debg_apt_tx.send(msg).expect("Failed to send initialize message to debug adapter");
}

pub fn handle_initialized_event(msg: json::JsonValue, ctx: &mut Context) {
    //This is where we'd set the breakpoints
    //Breakpoints hardcoded for now; TODO: receive breakpoints from editor.
    let break_msg = object!{
        "type": "request",
        "seq": ctx.next_req_id(),
        "command": "setBreakpoints",
        "arguments": {
            "source": {
                "name": "test",
                "path": "/home/jdugan/projects/kak_plugins/kak-dap/demo/python/test.py"
            },
            "breakpoints": [
                {
                    "line": 10
                }
            ]
        }
    };
    //Send it to the debug adapter
    ctx.debg_apt_tx.send(break_msg).expect("Failed to send setBreakpoints message to debug adapter");
}

pub fn handle_initialize_response(msg: json::JsonValue, ctx: &mut Context) {
    //Since debugpy uses "late case" initialization (https://github.com/microsoft/vscode/issues/4902#issuecomment-368583522),
    //we need to send the launch request before the breakpoints.
    let launch_msg = object!{
        "type": "request",
        "seq": ctx.next_req_id(),
        "command": "launch",
        "arguments": {
            "program": "/home/jdugan/projects/kak_plugins/kak-dap/demo/python/test.py",
            "args": [],
            "stopOnEntry": true,
            "console": "externalTerminal",
            "debugOptions": [],
            "cwd": "/home/jdugan/projects/kak_plugins/kak-dap/demo/python"
        }
    };
    //Send it to the debug adapter
    ctx.debg_apt_tx.send(launch_msg).expect("Failed to send initialize message to debug adapter");
    
}

pub fn handle_set_breakpoint_response(msg: json::JsonValue, ctx: &mut Context) {
    //For now, we will just set the one breakpoint.
    //Now, send the configurationDone request.
    //
    let launch_msg = object!{
        "type": "request",
        "seq": ctx.next_req_id(),
        "command": "configurationDone",
    };
    //Send it to the debug adapter
    ctx.debg_apt_tx.send(launch_msg).expect("Failed to send initialize message to debug adapter");
}

pub fn handle_run_in_terminal_request(msg: json::JsonValue, ctx: &mut Context) {
    let seq = &msg["seq"];
    ctx.last_adapter_seq = seq.to_string().parse::<u64>().unwrap();
    let args = &msg["arguments"]["args"];
    println!("{}", args.to_string());
    //Extract the program we need to run
    let mut cmd = "dap-run-in-terminal ".to_string();
    let args_members = args.members();
    for val in args_members {
        cmd.push_str("\"");
        cmd.push_str(&val.to_string());
        cmd.push_str("\" ");
    }
    println!("{}", cmd);
    kakoune::kak_command(cmd, &ctx);
}
