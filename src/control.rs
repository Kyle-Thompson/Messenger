

use net_lib::Net;
use io_lib::IOHandler;

pub struct Control {
    net: Net,
    io: IOHandler,
}

impl Control {

    pub fn new() -> Control {
        Control{
            net: Net::new(),
            io: IOHandler::new(),
        }
    }

    pub fn start(&mut self) {

    }
}
