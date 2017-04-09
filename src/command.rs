use std::sync::mpsc::channel;

use io_lib::IOHandler;
use net_lib::*;
use state::*;

pub fn handle(io: &IOHandler, net: &Net, state: &State, user: &mut Option<User>, tokens: &[&str]) {
    let cmd: &str = tokens[0];
    let args: &[&str] = &tokens[1..];
    
    match cmd.trim() {
        "/login" => {
            *user = match login(&io, &net) {
                Ok(usr) => {io.print_log("success"); Some(usr)},
                Err(e) => {
                    io.print_error(&e);
                    None
                },
            };
        },
        "/register" => {
            *user = match register(&io, &net) {
                Ok(usr) => {io.print_log("success"); Some(usr)},
                Err(e) => {
                    io.print_error(&e);
                    None
                },
            };
        },
        "/connect" => {
            if let Err(e) = connect(args, &net, &state) {
                io.print_error(&e);
            }
        },

        _ => {
            io.print_error("Command not recognized");
        },
    }

}

fn login(io: &IOHandler, net: &Net) -> Result<User, String> {
    let mut username: String = "".to_string();
    let mut password: String = "".to_string();
    io.read_prompted_line(&mut username, "Username: ");
    io.read_prompted_line(&mut password, "Password: ");

    let (sender, receiver) = channel();

    net.add_message(
        MessageContainer::new(
            Message::new(
                MessageType::Login{
                    username: username,
                    password: password
                },
                vec![Net::server_addr().to_string()]
            ),
            Some(sender)
        )
    );

    let res = match receiver.recv().unwrap() {
        Ok(res) => res,
        Err(e) => return Err(e.to_string()),
    };

    if let MessageType::Response(res) = res.unwrap().msg_type {
        match res {
            ResponseType::User(u) => Ok(u),
            ResponseType::Error(e) => Err(e),
            _ => Err("Something went wrong".to_string())
        }
    } else {
        Err("Reply was not of type 'Response'. Whut?".to_string())
    }
}

fn register(io: &IOHandler, net: &Net) -> Result<User, String>{
    let mut username: String = "".to_string();
    let mut password: String = "".to_string();
    io.read_prompted_line(&mut username, "Username: ");
    io.read_prompted_line(&mut password, "Password: ");

    let (sender, receiver) = channel();

    net.add_message(MessageContainer::new(
            Message::new(
                MessageType::Register{
                    username: username,
                    password: password
                },
                vec![Net::server_addr().to_string()]
            ),
        Some(sender)
    ));

    let res = match receiver.recv().unwrap() {
        Ok(res) => res,
        Err(e) => return Err(e.to_string()),
    };

    if let MessageType::Response(res) = res.unwrap().msg_type {
        match res {
            ResponseType::User(u) => Ok(u),
            ResponseType::Error(e) => Err(e),
            _ => Err("Something went wrong".to_string())
        }
    } else {
        Err("Reply was not of type 'Response'. Whut?".to_string())
    }
}

fn connect(args: &[&str], net: &Net, state: &State) -> Result<(), String>{
    let o_user = args[0];
    let route: Vec<String> = match net.get_route(&o_user) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let conv = Conversation::new(User{handle:o_user.to_string(), route:route});
    let conv_id = conv.get_id();
    state.add_conversation(conv);
    state.set_current_conversation(Some(conv_id));
    Ok(())
}

