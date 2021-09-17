use crate::context::*;
use crate::debug_adapter_comms;
use crate::types::{Scope,Variable};

use json::object;

//Handles the "scopes" response.
pub fn handle_scopes_response(msg: json::JsonValue, ctx: &mut Context) {
    //Update the "scopes" array in the context
    ctx.scopes.clear();
    let scopes_members = msg["body"]["scopes"].members();
    for val in scopes_members {
        let value = val.clone();
        //Enter this scope in the scopes array
        let scope = Scope {
            variable_reference: val["variablesReference"].to_string().parse::<u64>().unwrap(),
            line_no: 0,
            contents: value,
        };
        ctx.scopes.push(scope);
        //Request variables for this scope
        let var_ref = val["variablesReference"].to_string().parse::<u64>().unwrap();
        let var_args = object!{
            "variablesReference": var_ref
        };
        ctx.var_reqs += 1;
        debug_adapter_comms::do_request("variables".to_string(), var_args, ctx);
    }
}

//Handles the "variables" response.
pub fn handle_variables_response(msg: json::JsonValue, ctx: &mut Context) {
    ctx.var_reqs -= 1;
    //Find the variables request that spawned this response
    let cur_requests = &ctx.cur_requests.clone();
    let val_req = cur_requests.into_iter().find(|x| &x["seq"] == &msg["request_seq"]).unwrap();

    //Print every variable in the variables buffer
    let variables = &msg["body"]["variables"];
    let variables_members = variables.members();
    for val in variables_members {
        let val_cln = val.clone();
        //Construct an Expandable instance containing this variable's properties
        let variable = Variable {
            variable_reference: val["variablesReference"].to_string().parse::<u64>().unwrap(),
            par_variable_reference: val_req["arguments"]["variablesReference"].to_string().parse::<u64>().unwrap(),
            line_no: 0,
            contents: val_cln,
        };
    }
}

//Handles the "expand" command from the editor.
pub fn expand_variable(cmd: &String, ctx: &mut Context) {
    //TODO: implement.
}
