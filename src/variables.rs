use crate::context::*;
use crate::debug_adapter_comms;
use crate::types::{Scope,Variable};
use crate::kakoune;

use json::object;

//Handles the "scopes" response.
pub fn handle_scopes_response(msg: json::JsonValue, ctx: &mut Context) {
    //Update the "scopes" array in the context
    ctx.scopes.clear();
    ctx.variables.clear();
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

    //Loop over every variable in this response
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
        ctx.variables.push(variable);
    }
    // If we've serviced all pending variable requests, render the scopes and variables in the variables buffer
    if ctx.var_reqs == 0 {
        serialize_variables(ctx);
    }
}

//Constructs the command that renders all the scopes and variables in the variables buffer.
pub fn serialize_variables(ctx: &mut Context) {
    let mut cmd = "dap-show-variables 'Variables:\n\n".to_string();
    let mut cmd_val = "".to_string();
    for scope in &ctx.scopes {
        let scope_name = &scope.contents["name"];
        cmd_val.push_str(&"Scope: ".to_string());
        cmd_val.push_str(&scope_name.to_string());
        cmd_val.push_str("\n");
        // First confirm that this scope has child variables
        let mut has_child = false;
        for var in &ctx.variables {
            if var.par_variable_reference == scope.variable_reference {
                has_child = true;
                break;
            }
        }
        if has_child {
            let val = serialize_variable(ctx, scope.variable_reference, 4);
            cmd_val.push_str(&val);
        }
    }
    cmd.push_str(&kakoune::editor_escape(&cmd_val));
    cmd.push_str("'");
    kakoune::kak_command(cmd, ctx);
}

//Constructs the command that renders all the child variables of the given variable reference in the variables buffer.
pub fn serialize_variable(ctx: &Context, par_ref: u64, indent: u64) -> String {
    let mut val = "".to_string();
    for var in &ctx.variables {
        if var.par_variable_reference == par_ref {
            for _i in 0..indent {
                val.push_str(" ");
            }
            //If this variable is expandable
            if var.variable_reference > 0 {
                let mut icon = "+";
                //Determine if this variable has any child variables currently
                for varr in &ctx.variables {
                    if varr.par_variable_reference == var.variable_reference {
                        icon = "-";
                        break;
                    }
                }
                val.push_str(&format!("{} ", icon));
            }
            val.push_str("<");
            val.push_str(&var.variable_reference.to_string());
            val.push_str("> ");
            val.push_str(&var.contents["name"].to_string());
            val.push_str(" (");
            val.push_str(&var.contents["type"].to_string());
            val.push_str("): ");
            val.push_str(&var.contents["value"].to_string());
            val.push_str("\n");
            if var.variable_reference > 0 {
                val.push_str(&serialize_variable(ctx, var.variable_reference, indent + 4));
            }
        }
    }
    val
}

//Handles the "expand" command from the editor.
pub fn expand_variable(cmd: &String, ctx: &mut Context) {
    let mut var = cmd.to_string();
    var = var.trim().to_string();
    //If the string starts with a '<', this is an expandable variable
    let first_char = var.chars().next().unwrap();
    if first_char == '<' {
        //Extract the variable reference
        var = var[1..].to_string();
        let var_ref = var.parse::<u64>().unwrap();
        //If the variables list contains any child variables of this variable reference, then it's expanded
        let mut is_expanded = false;
        for varr in &ctx.variables {
            if varr.par_variable_reference == var_ref {
                is_expanded = true;
                break;
            }
        }
        //If this variable isn't expanded, then expand it
        if !is_expanded {
            let var_args = object!{
                "variablesReference": var_ref
            };
            ctx.var_reqs += 1;
            debug_adapter_comms::do_request("variables".to_string(), var_args, ctx);
        }
        //Otherwise, collapse it
        else {
            ctx.variables.retain(|x| x.par_variable_reference != var_ref);
            serialize_variables(ctx);
        }
    }
}
