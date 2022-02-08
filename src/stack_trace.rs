use crate::context::*;
use crate::debug_adapter_comms;
use crate::kakoune;

use json::object;

// Handles the "stopped" event.
pub fn handle_stopped_event(_msg: json::JsonValue, ctx: &mut Context) {
    // Send a stack trace request
    let stack_trace_args = object! {
        "threadId": 1
    };
    debug_adapter_comms::do_request("stackTrace", stack_trace_args, ctx);
}

// Handles the "stackTrace" response.
pub fn handle_stack_trace_response(msg: json::JsonValue, ctx: &mut Context) {
    let frames = &msg["body"]["stackFrames"];
    // Get first stack frame to obtain current execution location
    let frame = &frames[0];
    let line = &frame["line"];
    let file = &frame["source"]["path"];
    // Construct Kakoune command to jump to location
    let mut cmd = "dap-stack-trace ".to_string();
    cmd.push_str(&line.to_string());
    cmd.push_str(" ");
    cmd.push_str(&file.to_string());
    cmd.push_str(" ");
    cmd.push_str("'Stack Trace:\n\n");
    // Add contents to push to stacktrace buffer
    let frame_members = frames.members();
    for val in frame_members {
        let id = &val["id"];
        let name = &val["name"];
        // Get the filename from the path (name is not guaranteed to exist)
        let slash_index = val["source"]["path"].to_string().rfind("/").unwrap();
        let path = &val["source"]["path"].to_string();
        let file = path.get((slash_index + 1)..).unwrap();
        let line = &val["line"];
        cmd.push_str(&format!("{}: {}@{}:{}", id, name, file, line));
        cmd.push_str("\n");
    }
    cmd.push_str("'");
    kakoune::kak_command(&cmd, &ctx);
    // Send a Scopes message to kickstart retrieving the variables
    let id = frames[0]["id"].to_string().parse::<u64>().unwrap();
    ctx.cur_stack = id;
    let scopes_args = object! {
        "frameId": id,
    };
    debug_adapter_comms::do_request("scopes", scopes_args, ctx);
}
