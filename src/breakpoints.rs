use crate::context::*;
use crate::debug_adapter_comms;

use json::{JsonValue, object};

pub fn process_breakpoints(ctx: &mut Context, breakpoints: JsonValue) {
    // If we have no breakpoints, don't do anything
    if breakpoints["breakpoints"].is_empty() {
        return;
    }
    let raw_breakpoints = breakpoints["breakpoints"].to_string();
    // Split passed value along spaces
    let split = raw_breakpoints.split_whitespace();
    for val in split {
        // Get the filepath and line number
        let bar_index = val.chars().position(|c| c == '|').unwrap();
        let line_no = &val[0..bar_index].parse::<u64>().unwrap();
        let filepath = &val[bar_index + 1..];
        debug!("Filepath: {}, line_no: {}", filepath, line_no);
        // If we already have breakpoints in this filename
        if ctx.breakpoint_data.contains_key(filepath) {
            let mut new_vec = ctx.breakpoint_data.get(filepath).unwrap().clone();
            new_vec.push(*line_no);
            ctx.breakpoint_data.insert(filepath.to_string(), new_vec);
        }
        // Otherwise, create a new entry
        else {
            let mut new_vec = vec![];
            new_vec.push(*line_no);
            ctx.breakpoint_data.insert(filepath.to_string(), new_vec);
        }
    }
}

// Handles the "initialized" event.
pub fn handle_initialized_event(_msg: json::JsonValue, ctx: &mut Context) {
    // This is where we set the breakpoints
    let mut requests : Vec<json::JsonValue> = vec![];
    // Loop over the various source files we were sent
    for (source, lines) in &ctx.breakpoint_data {
        let mut breakpoints = json::JsonValue::new_array();
        // Ensure we get all the lines in each file
        for line in lines {
            breakpoints.push(object! {
                "line": *line,
            }).expect("Couldn't add breakpoint to list");
        }
        // Construct the actual request's arguments
        let mut break_args = object! {
            "source": {
                "path": source.to_string(),
            },
        };
        break_args["breakpoints"] = breakpoints;
        requests.push(break_args);
    }
    // Send all the breakpoint requests, one after another
    for req in requests {
        debug_adapter_comms::do_request("setBreakpoints", req, ctx);
    }
  
    // Now, send the configurationDone request.
    debug_adapter_comms::do_request("configurationDone", object! {}, ctx);
}
