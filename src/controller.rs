use std::sync::{Arc, Mutex};
use std::thread;

use json::{object, JsonValue};

use crate::breakpoints;
use crate::config;
use crate::context::*;
use crate::debug_adapter_comms;
use crate::general;
use crate::kakoune;
use crate::stack_trace;
use crate::variables;

pub fn start(session: &String, breakpoints: JsonValue) {
    let kakoune_rx = kakoune::start_kak_comms(session);
    let config : JsonValue;
    // Load the debug configuration
    let config_path = config::config_path();
    if config_path.is_none() {
        // No configuration found; bail
        error!("Couldn't find configuration");
        general::goodbye(session);
    }
    let pot_config = config::get_config(&config_path.unwrap());
    if pot_config.is_none() {
        error!("Invalid configuration");
        // Configuration isn't valid JSON; bail
        general::goodbye(session);
    }
    config = pot_config.unwrap();
    debug!("Debug configuration: {}", config.dump());
    // Begin communication with the debug adapter
    let mut adapter_args : Vec<String> = vec![];
    for val in config["adapter_args"].members() {
        adapter_args.push(val.to_string());
    }
    let (adapter_tx, adapter_rx) = debug_adapter_comms::debug_start(
        &config["adapter"].to_string(),
        &adapter_args,
    );

    let cxt_src = Arc::new(Mutex::new(Context::new(adapter_tx, session.to_string())));

    // Debug adapter response handling thread
    let ctx = Arc::clone(&cxt_src);
    thread::spawn(move || {
        for msg in adapter_rx {
            let mut ctx = ctx.lock().expect("Failed to lock context");
            if msg["type"].to_string() == "response" {
                let msg_cln = msg.clone();
                let ctx_cln = &mut ctx;
                handle_adapter_response(msg, ctx_cln);
                // Find the request that spawned this response and remove it from the pending requests
                ctx_cln
                    .cur_requests
                    .retain(|x| &x["seq"] != &msg_cln["request_seq"]);
            } else if msg["type"].to_string() == "event" {
                handle_adapter_event(msg, &mut ctx);
            } else if msg["type"].to_string() == "request" {
                // Right now, there is only one "reverse request" - runInTerminal.
                // Therefore, we simply handle that request.
                general::handle_run_in_terminal_request(msg, &mut ctx);
            }
        }
    });

    // Set dap_running value in Kakoune; process breakpoints; initialize the debug adapter
    let ctx = Arc::clone(&cxt_src);
    {
        let mut ctx = ctx.lock().expect("Failed to lock context");
        ctx.debug_cfg = config;
        kakoune::kak_command("set-option global dap_running true", &ctx.session);
        breakpoints::process_breakpoints(&mut ctx, breakpoints);
        general::initialize(&mut ctx);
    }

    // Handle messages from Kakoune
    for msg in kakoune_rx {
        let mut ctx = ctx.lock().expect("Failed to lock context");
        parse_cmd(msg, &mut ctx);
    }
}

// Handle events from the debug adapter.
pub fn handle_adapter_event(msg: json::JsonValue, ctx: &mut Context) {
    match msg["event"].to_string().as_str() {
        "exited" => general::goodbye(&ctx.session),
        "initialized" => breakpoints::handle_initialized_event(msg, ctx),
        "output" => general::output(msg, ctx),
        "stopped" => stack_trace::handle_stopped_event(msg, ctx),
        "terminated" => general::goodbye(&ctx.session),
        _ => (),
    };
}

// Handle responses from the debug adapter.
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

// Handle commands from Kakoune.
pub fn parse_cmd(cmd: json::JsonValue, ctx: &mut Context) {
    // Depending on the command given, act accordingly
    if cmd["cmd"] == "stop" {
        // We currently rely on the adapter terminating the debuggee once stdio streams are closed
        general::goodbye(&ctx.session);
    } else if cmd["cmd"] == "continue" {
        // Send a continue command to the debugger
        let continue_args = object! {
            "threadId": ctx.cur_thread
        };
        debug_adapter_comms::do_request("continue", &continue_args, ctx);
    } else if cmd["cmd"] == "next" {
        // Send a next command to the debugger
        let next_args = object! {
            "threadId": ctx.cur_thread
        };
        debug_adapter_comms::do_request("next", &next_args, ctx);
    } else if cmd["cmd"] == "pid" {
        // Send response to debug adapter
        debug_adapter_comms::do_response("runInTerminal", object! {}, ctx);
    } else if cmd["cmd"] == "stepIn" {
        // Send a stepIn command to the debugger
        let step_in_args = object! {
            "threadId": ctx.cur_thread
        };
        debug_adapter_comms::do_request("stepIn", &step_in_args, ctx);
    } else if cmd["cmd"] == "stepOut" {
        // Send a stepIn command to the debugger
        let step_out_args = object! {
            "threadId": ctx.cur_thread
        };
        debug_adapter_comms::do_request("stepOut", &step_out_args, ctx);
    } else if cmd["cmd"] == "evaluate" {
        // Extract the expression and send an "evaluate" command to the debugger
        let expr = cmd["args"].to_string();
        let eval_args = object! {
            "expression": expr,
            "frameId": ctx.cur_stack
        };
        debug_adapter_comms::do_request("evaluate", &eval_args, ctx);
    } else if cmd["cmd"] == "expand" {
        variables::expand_variable(&cmd["args"].to_string(), ctx);
    }
}
