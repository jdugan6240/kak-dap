use crate::context::*;
use crate::debug_adapter_comms;
use crate::kakoune;
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

//Get the string that represents the contents to place in the Variables buffer
/*pub fn serialize_variables(vars: &Vec<Expandable>, indent: u64) -> String {
    let mut value = "".to_owned();
    //Loop through every expandable in the list
    for val in vars.iter() {
        //Add whitespace to match indent
        for _n in 1..indent {
            value.push_str(" ");
        }
        let contents = &val.contents;
        //If this is a scope
        if !val.is_var {
            value.push_str(&"Scope: ".to_string());
            value.push_str(&contents["name"].to_string());
        }
        //If this is a variable
        else {
            value.push_str(&contents["name"].to_string());
            value.push_str(" (");
            value.push_str(&contents["type"].to_string());
            value.push_str("): ");
            value.push_str(&contents["value"].to_string());
        }
        value.push_str("\n");
        //If this Expandable has sub-variables
        if val.variable_reference != 0 {
            value.push_str(&serialize_variables(&val.variables, indent + 4));
        }
    }
    value
}*/

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
            par_variable_reference: msg["arguments"]["variablesReference"].to_string().parse::<u64>().unwrap(),
            line_no: 0,
            contents: val_cln,
        };
        //kakoune::print_debug(&val_req[.unwrap()["variablesReference"].to_string(), &ctx);
        //kakoune::print_debug(&val_req.unwrap().to_string(), &ctx);
        //Find the variable in the current heirarchy that has this variable reference
        
        /*let mut cmd = "dap-add-variable '".to_string();
        let mut icon = " ";
        if val["variablesReference"].to_string().parse::<u64>().unwrap() > 0 {
            icon = "+";
        }
        cmd.push_str(icon);
        cmd.push_str("<");
        cmd.push_str(&val["variablesReference"].to_string());
        cmd.push_str(">");
        cmd.push_str(" ");
        cmd.push_str(&val["name"].to_string());
        cmd.push_str(" (");
        cmd.push_str(&val["type"].to_string());
        cmd.push_str("): ");
        cmd.push_str(&val["value"].to_string());
        cmd.push_str("\n'");
        kakoune::kak_command(cmd, ctx);*/
    }
}

//Handles the "expand" command from the editor.
pub fn expand_variable(cmd: &String, ctx: &mut Context) {
    /*let var = cmd[7..].to_string();
    let var_num = var.trim().to_string().parse::<u64>().unwrap();
    //The resulting value is the variable reference
    //Now, run a variables request to get the resulting variable
    //kakoune::print_debug(&var.to_string(), ctx);
    let var_args = object!{
        "variablesReference": var_num,
    };
    debug_adapter_comms::do_request("variables".to_string(), var_args, ctx);*/

}
