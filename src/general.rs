use crate::context::*;
use crate::debug_adapter_comms;
use crate::kakoune;

use json::object;
use std::process;

// Initializes the debug adapter.
pub fn initialize(ctx: &mut Context) {
    // Construct the initialize request
    let initialize_args = object! {
        "adapterID": ctx.debug_cfg["adapterID"].to_string(),
        "linesStartAt1": true,
        "columnsStartAt1": true,
        "pathFormat": "path",
        "supportsRunInTerminalRequest": true,
    };
    debug_adapter_comms::do_request("initialize", &initialize_args, ctx);
}


// Handles the "initialize" response.
pub fn handle_initialize_response(_msg: json::JsonValue, ctx: &mut Context) {
    // We need to send the launch request before the breakpoints.
    // For background: https://github.com/microsoft/vscode/issues/4902
    let launch_args : &json::JsonValue = &ctx.debug_cfg["launch_args"];
    let launch_args_cln = launch_args.clone();
    debug_adapter_comms::do_request("launch", &launch_args_cln, ctx);
}

// Handles the "runInTerminal" request.
pub fn handle_run_in_terminal_request(msg: json::JsonValue, ctx: &mut Context) {
    // Get the sequence number of this request to send back later
    let seq = &msg["seq"];
    ctx.last_adapter_seq = seq.to_string().parse::<u64>().unwrap();
    // Extract the program we need to run
    let args = &msg["arguments"]["args"];
    let mut cmd = "dap-run-in-terminal ".to_string();
    let args_members = args.members();
    for val in args_members {
        cmd.push_str("\"");
        cmd.push_str(&val.to_string());
        cmd.push_str("\" ");
    }
    kakoune::kak_command(&cmd, &ctx.session);
}

//Handles the "evaluate" response.
pub fn handle_evaluate_response(msg: json::JsonValue, ctx: &mut Context) {
    //Get the result and type
    let result = &msg["body"]["result"];
    let typ = &msg["body"]["type"];

    //Send it to Kakoune for processing
    let mut cmd = "dap-evaluate-response ' ".to_string();
    cmd.push_str(&kakoune::editor_escape(&result.to_string()));
    cmd.push_str(" ' ' ");
    cmd.push_str(&kakoune::editor_escape(&typ.to_string()));
    cmd.push_str(" '");
    kakoune::kak_command(&cmd, &ctx.session);
}

//Tries to end kak-dap gracefully.
pub fn goodbye(session: &str) {
    kakoune::kak_command("try %{ eval -client %opt{jumpclient} %{ dap-reset-location }}", session);
    kakoune::kak_command("try %{ eval -client %opt{jumpclient} %{ dap-takedown-ui }}", session);
    kakoune::kak_command("set-option global dap_running false", session);
    kakoune::clean_socket(&session.to_string());
    process::exit(0);
}
