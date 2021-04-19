use std::process;
use crossbeam_utils::thread;

use crate::kakoune;
use crate::debug_adapter_comms;

pub fn start(session: &String) {
    //Begin communication with the debug adapter
    //Debug adapter hardcoded for now; TODO: make configurable
    let (adapter_tx, adapter_rx) = debug_adapter_comms::debug_start("node", &["~/.vscode-oss/extensions/webfreak.debug-0.25.0/out/src/lldb.js".to_string()]);
    //Event loop
    thread::scope(|s| {
        s.spawn(|_| {
            for msg in adapter_rx {
                //TODO: parse and handle messages from the debug adapter
                kakoune::print_debug(msg.to_string(), session);
            }
        });
    });
}

pub fn parse_cmd(command: &String, session: &String) {
    //Trim the newline from the command
    let cmd = command.trim();

    //Depending on the command given, act accordingly
    if cmd == "stop" {
        kakoune::kak_command("set-option global dap_running false".to_string(), session);
        process::exit(0);
    }
}
