use std::process;
use std::thread;
//use crossbeam_utils::thread;

use crate::kakoune;
use crate::debug_adapter_comms;

pub fn start(session: &String) {
    let kakoune_rx = kakoune::start_kak_comms();
    //Begin communication with the debug adapter
    //Debug adapter hardcoded for now; TODO: make configurable
    let (_adapter_tx, adapter_rx) = debug_adapter_comms::debug_start("python", &["/home/jdugan/debugpy/src/debugpy/adapter".to_string()]);

    let my_session: String = session.clone();

    thread::spawn(move || {
        for msg in adapter_rx {
            //TODO: parse and handle messages from the debug adapter
            //println!("{}", msg.to_string());
            kakoune::print_debug(&msg.to_string(), &my_session);
        }
    });
    //Event loop
    /*thread::scope(|s| {
        s.spawn(|_| {
            for msg in adapter_rx {
                //TODO: parse and handle messages from the debug adapter
                kakoune::print_debug(msg.to_string(), session);
            }
        });
    });*/
    //Main loop
    /*for msg in kakoune_rx {
        parse_cmd(msg, &session);
    }*/
    /*loop {
        let msg = kakoune_rx.recv().unwrap();
        parse_cmd(msg, &session);
    }*/
    for msg in kakoune_rx {
        parse_cmd(msg.to_string(), &session);
    }
}

pub fn parse_cmd(command: String, session: &String) {
    //Trim the newline from the command
    let cmd = command.trim();

    //Depending on the command given, act accordingly
    if cmd == "stop" {
        kakoune::kak_command("set-option global dap_running false".to_string(), session);
        process::exit(0);
    }
}
