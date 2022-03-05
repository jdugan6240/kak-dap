use crossbeam_channel::Sender;
use std::collections::HashMap;

use crate::types::{Scope, Variable};

use json::{object, JsonValue};

// Struct with which to carry around our "global" variables
pub struct Context {
    // Handle to write to the debug adapter
    pub debg_apt_tx: Sender<json::JsonValue>,
    // The sequence ID of the next request
    pub cur_req_id: u64,
    // The Kakoune session that spawned us
    pub session: String,
    // The sequence ID of the last reverseRequest from the adapter
    pub last_adapter_seq: u64,
    // The thread that last triggered the Stopped event
    pub cur_thread: u64,
    // The scopes found at the last Stopped event
    pub scopes: Vec<Scope>,
    // The variables currently stored in the variable heirarchy
    pub variables: Vec<Variable>,
    // The number of Variables requests we still need to service
    pub var_reqs: u64,
    // The current stack frame.
    pub cur_stack: u64,
    // The requests we have sent to the adapter
    pub cur_requests: Vec<json::JsonValue>,
    // The breakpoints passed to the kak-dap session
    pub breakpoint_data: HashMap<String, Vec<u64>>,
    // The debug configuration
    pub debug_cfg: JsonValue
}

impl Context {
    pub fn new(debg_apt_tx: Sender<json::JsonValue>, session: String) -> Self {
        Context {
            debg_apt_tx: debg_apt_tx,
            cur_req_id: 0,
            session: session,
            last_adapter_seq: 0,
            cur_thread: 1,
            scopes: vec![],
            variables: vec![],
            var_reqs: 0,
            cur_stack: 0,
            cur_requests: vec![],
            breakpoint_data: HashMap::new(),
            debug_cfg: object!{},
        }
    }

    pub fn next_req_id(&mut self) -> u64 {
        // Increment the current ID and return what it was before
        self.cur_req_id += 1;
        self.cur_req_id - 1
    }
}
