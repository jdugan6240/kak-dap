use std::sync::{Arc, Mutex};
use std::thread;

use json::{object, JsonValue};

use crate::context::*;
use crate::debug_adapter_comms;
use crate::general;
use crate::kakoune;
use crate::stack_trace;
use crate::variables;

pub fn process_breakpoints(ctx: &mut Context, breakpoints: JsonValue) {
    let raw_breakpoints = breakpoints["breakpoints"].to_string();
    //Split passed value along spaces
    let split = raw_breakpoints.split_whitespace();
    for val in split {
        //Get the filepath and line number
        let bar_index = val.chars().position(|c| c == '|').unwrap();
        let line_no = &val[0..bar_index].parse::<u64>().unwrap();
        let filepath = &val[bar_index + 1..];
        debug!("Filepath: {}, line_no: {}", filepath, line_no);
        //If we already have breakpoints in this filename
        if ctx.breakpoint_data.contains_key(filepath) {
            let mut new_vec = ctx.breakpoint_data.get(filepath).unwrap().clone();
            new_vec.push(*line_no);
            ctx.breakpoint_data.insert(filepath.to_string(), new_vec);
        }
        //Otherwise, create a new entry
        else {
            let mut new_vec = vec![];
            new_vec.push(*line_no);
            ctx.breakpoint_data.insert(filepath.to_string(), new_vec);
        }
    }
}

pub fn start(session: &String, breakpoints: JsonValue) {
    let kakoune_rx = kakoune::start_kak_comms(session);
    //Begin communication with the debug adapter
    //Debug adapter hardcoded for now; TODO: make configurable
    let (adapter_tx, adapter_rx) = debug_adapter_comms::debug_start(
        "python",
        &["/home/jdugan/debugpy/src/debugpy/adapter".to_string()],
    );

    let cxt_src = Arc::new(Mutex::new(Context::new(adapter_tx, session.to_string())));

    //Set dap_running option in Kakoune session
    {
        let ctx = cxt_src.lock().expect("Failed to lock context");
        kakoune::kak_command("set-option global dap_running true".to_string(), &ctx);
    }

    //Process the breakpoints
    let ctx = Arc::clone(&cxt_src);
    {
        let mut ctx = ctx.lock().expect("Failed to lock context");
        let ctx_cln = &mut ctx;
        process_breakpoints(ctx_cln, breakpoints);
    }

    //Debug adapter response handling thread
    let ctx = Arc::clone(&cxt_src);
    thread::spawn(move || {
        for msg in adapter_rx {
            let mut ctx = ctx.lock().expect("Failed to lock context");
            if msg["type"].to_string() == "response" {
                let msg_cln = msg.clone();
                let ctx_cln = &mut ctx;
                handle_adapter_response(msg, ctx_cln);
                //Find the request that spawned this response and remove it from the pending requests
                ctx_cln
                    .cur_requests
                    .retain(|x| &x["seq"] != &msg_cln["request_seq"]);
            } else if msg["type"].to_string() == "event" {
                handle_adapter_event(msg, &mut ctx);
            } else if msg["type"].to_string() == "request" {
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
        parse_cmd(msg, &mut ctx);
    }
}

//Handle events from the debug adapter.
pub fn handle_adapter_event(msg: json::JsonValue, ctx: &mut Context) {
    match msg["event"].to_string().as_str() {
        "exited" => general::goodbye(ctx),
        "initialized" => general::handle_initialized_event(msg, ctx),
        "stopped" => stack_trace::handle_stopped_event(msg, ctx),
        "terminated" => general::goodbye(ctx),
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
pub fn parse_cmd(cmd: json::JsonValue, ctx: &mut Context) {
    //Depending on the command given, act accordingly
    if cmd["cmd"] == "stop" {
        //We currently rely on the adapter terminating the debuggee once stdio streams are closed
        general::goodbye(ctx);
    } else if cmd["cmd"] == "continue" {
        //Send a continue command to the debugger
        let continue_args = object! {
            "threadId": 1
        };
        debug_adapter_comms::do_request("continue".to_string(), continue_args, ctx);
    } else if cmd["cmd"] == "next" {
        //Send a next command to the debugger
        let next_args = object! {
            "threadId": 1
        };
        debug_adapter_comms::do_request("next".to_string(), next_args, ctx);
    } else if cmd["cmd"] == "pid" {
        //Send response to debug adapter
        debug_adapter_comms::do_response("runInTerminal".to_string(), object! {}, ctx);
    } else if cmd["cmd"] == "stepIn" {
        //Send a stepIn command to the debugger
        let step_in_args = object! {
            "threadId": 1
        };
        debug_adapter_comms::do_request("stepIn".to_string(), step_in_args, ctx);
    } else if cmd["cmd"] == "stepOut" {
        //Send a stepIn command to the debugger
        let step_out_args = object! {
            "threadId": 1
        };
        debug_adapter_comms::do_request("stepOut".to_string(), step_out_args, ctx);
    } else if cmd["cmd"] == "evaluate" {
        //Extract the expression and send an "evaluate" command to the debugger
        let expr = cmd["args"].to_string();
        let eval_args = object! {
            "expression": expr,
            "frameId": ctx.cur_stack
        };
        debug_adapter_comms::do_request("evaluate".to_string(), eval_args, ctx);
    } else if cmd["cmd"] == "expand" {
        variables::expand_variable(&cmd["args"].to_string(), ctx);
    }
}
