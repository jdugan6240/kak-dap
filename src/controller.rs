use std::process;

use crate::kakoune;

pub fn parse_cmd(command: &String, session: &String) {
    //Trim the newline from the command
    let cmd = command.trim();

    if cmd == "stop" {
        kakoune::kak_command("set-option global dap_running false".to_string(), session);
        process::exit(0);
    }
}
