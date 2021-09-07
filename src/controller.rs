use std::process;
use std::thread;
use std::sync::{Arc, Mutex};

use json::object;

use crate::kakoune;
use crate::debug_adapter_comms;
use crate::context::*;
use crate::general;
use crate::stack_trace;
use crate::variables;

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
            if msg["type"].to_string() == "response" {
                let msg_cln = msg.clone();
                let ctx_cln = &mut ctx;
                handle_adapter_response(msg, ctx_cln);
                //Find the request that spawned this response and remove it from the pending requests
                ctx_cln.cur_requests.retain(|x| &x["seq"] != &msg_cln["request_seq"]);
            }
            else if msg["type"].to_string() == "event" {
                handle_adapter_event(msg, &mut ctx);
            }
            else if msg["type"].to_string() == "request" {
                //Right now, there is only one "reverse request" - runInTerminal.
                //Therefore, we simply handle that request.
                general::handle_run_in_terminal_request(msg, &mut ctx);
            }
        }
    });
    //Initialize the debug adapter
    let ctx = Arc::clone(&cxt_src);
    {
        let mut ctx = ctx.lock().expect("Failed to lock context");
        general::initialize(&mut ctx);
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
        "initialized" => general::handle_initialized_event(msg, ctx),
        "stopped" => stack_trace::handle_stopped_event(msg, ctx),
        _ => (),
    };
}

//Handle responses from the debug adapter.
pub fn handle_adapter_response(msg: json::JsonValue, ctx: &mut Context) {
    match msg["command"].to_string().as_str() {
        "initialize" => general::handle_initialize_response(msg, ctx),
        "stackTrace" => stack_trace::handle_stack_trace_response(msg, ctx),
        "scopes" => variables::handle_scopes_response(msg, ctx),
        "variables" => variables::handle_variables_response(msg, ctx),
        "evaluate" => general::handle_evaluate_response(msg, ctx),
        _ => (),
    };
}

//Handle commands from Kakoune.
pub fn parse_cmd(command: String, ctx: &mut Context) {
    //kakoune::print_debug(&command, ctx);
    //Trim the newline from the command
    let cmd = command.trim();

    //Depending on the command given, act accordingly
    if cmd == "stop" {
        kakoune::kak_command("set-option global dap_running false".to_string(), ctx);
        process::exit(0);
    }
    else if cmd == "continue" {
        //Send a continue command to the debugger
        let continue_args = object!{
            "threadId": 1
        };
        debug_adapter_comms::do_request("continue".to_string(), continue_args, ctx);
    }
    else if cmd == "next" {
        //Send a next command to the debugger
        let next_args = object!{
            "threadId": 1
        };
        debug_adapter_comms::do_request("next".to_string(), next_args, ctx);
    }
    else if cmd == "pid" {
        //Send response to debug adapter
        debug_adapter_comms::do_response("runInTerminal".to_string(), object!{}, ctx);
    }
    else if cmd == "stepIn" {
        //Send a stepIn command to the debugger
        let step_in_args = object!{
            "threadId": 1
        };
        debug_adapter_comms::do_request("stepIn".to_string(), step_in_args, ctx);
    }
    else if cmd == "stepOut" {
        //Send a stepIn command to the debugger
        let step_out_args = object!{
            "threadId": 1
        };
        debug_adapter_comms::do_request("stepOut".to_string(), step_out_args, ctx);
    }
    else if cmd.starts_with("evaluate") {
        //Extract the expression and send an "evaluate" command to the debugger
        let expr = cmd[9..].to_string();
        let eval_args = object!{
            "expression": expr,
            "frameId": ctx.cur_stack
        };
        debug_adapter_comms::do_request("evaluate".to_string(), eval_args, ctx);
    }
    else if cmd.starts_with("expand") {
        variables::expand_variable(&command, ctx);
    }
}

