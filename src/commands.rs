
use io_lib::IOHandler;
use net_lib::Net;
use state::User;
use state::State;


pub fn cmd_login(args: &Vec<&str>, io: &IOHandler, net: &Net) -> Result<User, &'static str> {
    let mut username: String = "".to_string();
    let mut password: String = "".to_string();
    io.read_prompted_line(&mut username, "Username: ");
    io.read_prompted_line(&mut password, "Password: ");
    Err("yolo")
}

pub fn cmd_register(args: &Vec<&str>, io: &IOHandler, net: &Net) -> Result<User, &'static str>{
    Err("yolo")
}

pub fn cmd_connect(args: &Vec<&str>, net: &Net, state: &State) -> Result<User, &'static str>{
    Err("yolo")
}

