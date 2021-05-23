use crossbeam_channel::Sender;

//Struct with which to carry around our "global" variables
pub struct Context {
    pub debg_apt_tx: Sender<json::JsonValue>,
    pub cur_req_id: u64,
    pub session: String,
    pub last_adapter_seq: u64,
    pub cur_thread: u64,
}

impl Context {
    pub fn new(debg_apt_tx: Sender<json::JsonValue>, session: String) -> Self {
        Context {
            debg_apt_tx: debg_apt_tx,
            cur_req_id: 0,
            session: session,
            last_adapter_seq: 0,
            cur_thread: 0,
        }
    }

    pub fn next_req_id(&mut self) -> u64 {
        // Increment the current ID and return what it was before
        self.cur_req_id += 1;
        self.cur_req_id - 1
    }
}
