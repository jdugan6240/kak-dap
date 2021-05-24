use crate::context::*;
use crate::debug_adapter_comms;
use crate::kakoune;

use json::object;

//Handles the "scopes" response.
pub fn handle_scopes_response(msg: json::JsonValue, ctx: &mut Context) {
    //Grab the variable reference from the local scope
    //TODO: return results from all scopes
    let scope = &msg["body"]["scopes"][0];
    let var_ref = scope["variablesReference"].to_string().parse::<u64>().unwrap();
    //Send a Variables message
    let var_args = object!{
        "variablesReference": var_ref
    };
    debug_adapter_comms::do_request("variables".to_string(), var_args, ctx);
    //Clear the "variables" buffer
    kakoune::kak_command("dap-clear-variables".to_string(), ctx);
}

//Handles the "variables" response.
pub fn handle_variables_response(msg: json::JsonValue, ctx: &mut Context) {
    //Print every variable in the variables buffer
    let variables = &msg["body"]["variables"];
    let variables_members = variables.members();
    for val in variables_members {
        let mut cmd = "dap-add-variable '".to_string();
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
        kakoune::kak_command(cmd, ctx);
    }
}

//Handles the "expand" command from the editor.
pub fn expand_variable(cmd: &String, ctx: &mut Context) {
    let var = cmd[7..].to_string();
    let var_num = var.trim().to_string().parse::<u64>().unwrap();
    //The resulting value is the variable reference
    //Now, run a variables request to get the resulting variable
    //kakoune::print_debug(&var.to_string(), ctx);
    let var_args = object!{
        "variablesReference": var_num,
    };
    debug_adapter_comms::do_request("variables".to_string(), var_args, ctx);

}
