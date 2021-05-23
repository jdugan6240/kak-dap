use crate::context::*;
use crate::debug_adapter_comms;
use crate::kakoune;

use json::object;

//Handles the "stopped" event.
pub fn handle_stopped_event(_msg: json::JsonValue, ctx: &mut Context) {
    //Send a stack trace request
    let stack_trace_args = object!{
        "threadId": 1
    };
    debug_adapter_comms::do_request("stackTrace".to_string(), stack_trace_args, ctx);
}

pub fn handle_stack_trace_response(msg: json::JsonValue, ctx: &mut Context) {
    let frames = &msg["body"]["stackFrames"];
    //Only interested in first frame for now
    //TODO: Iterate over returned stack frames to populate stack trace buffer
    let frame = &frames[0];
    let line = &frame["line"];
    let file = &frame["source"]["path"];
    //Construct Kakoune command to jump to location
    let mut cmd = "dap-stack-trace ".to_string();
    cmd.push_str(&line.to_string());
    cmd.push_str(" ");
    cmd.push_str(&file.to_string());
    kakoune::kak_command(cmd, &ctx);
}
