use io_lib::IOHandler;
use net_lib::Net;
use state::User;
use state::State;

pub fn handle(io: &IOHandler, net: &Net, state: &State, user: &mut Option<User>, tokens: &[&str]) {
    let cmd: &str = tokens[0];
    let args: &[&str] = &tokens[1..];
    match cmd {
        "/login" => {
            *user = match login(args, &io, &net) {
                Ok(usr) => Some(usr),
                Err(e) => {
                    io.print_error(e);
                    None
                ,
            };
        },
        "/register" => {
            *user = match register(args, &io, &net) {
                Ok(usr) => Some(usr),
                Err(e) => {
                    io.print_error(e);
                    None
                },
            };
        },
        "/connect" => {
            if let Err(e) = connect(args, &net, &state) {
                io.print_error(e);
            }
        },

        _ => {
            io.print_error("Command not recognized");
        },
    }

}

fn login(args: &[&str], io: &IOHandler, net: &Net) -> Result<User, &'static str> {
    let mut username: String = "".to_string();
    let mut password: String = "".to_string();
    io.read_prompted_line(&mut username, "Username: ");
    io.read_prompted_line(&mut password, "Password: ");
    let sup = Net::server_addr();
    Err("yolo")
}

fn register(args: &[&str], io: &IOHandler, net: &Net) -> Result<User, &'static str>{
    Err("yolo")
}

fn connect(args: &[&str], net: &Net, state: &State) -> Result<User, &'static str>{
    Err("yolo")
}

