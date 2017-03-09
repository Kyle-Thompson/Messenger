//use std::collections::{HashMap, HashSet};

use net_lib::Net;
use io_lib::IOHandler;
use state::State;

pub struct Control {
    net: Net,
    io: IOHandler,
    state: State,
}

impl Control {

    pub fn new() -> Control {
        Control{
            net: Net::new(),
            io: IOHandler::new(),
            state: State::new(),
        }
    }

    pub fn start(&mut self) {
        
    }
}
