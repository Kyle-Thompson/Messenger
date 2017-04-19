use std::sync::mpsc::channel;
use std::error::Error;

use io_lib::IOHandler;
use net_lib::*;
use state::*;

pub fn handle(io: &IOHandler, net: &Net, state: &State, user: &mut Option<User>, tokens: &[&str]) {
    let cmd: &str = tokens[0];
    let args: &[&str] = &tokens[1..];
    
    match cmd.trim() {
        "/login" => {
            *user = match login(&io, &net) {
                Ok(usr) => Some(usr),
                Err(e) => {
                    io.print_error(&e);
                    None
                },
            };
        },
        "/register" => {
            *user = match register(&io, &net) {
                Ok(usr) => Some(usr),
                Err(e) => {
                    io.print_error(&e);
                    None
                },
            };
        },
        "/connect" => {
            if let Err(e) = connect(args[0], &net, &state) {
                io.print_error(&e);
            }
        },
        "/leave" => {
            leave(&state, &io);
        }
        "/join" => {
            join(args[0], &state, &io);
        },
        "/list" => {
            list(&state, &io);
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

    let res = match receiver.recv() {
        Ok(res) => match res {
            Ok(res) => res,
            Err(e) => return Err(e.to_string()),
        },
        Err(e) => return Err("wtf".to_string() + e.description())
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

fn connect(o_user: &str, net: &Net, state: &State) -> Result<(), String>{
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

fn leave(state: &State, io: &IOHandler) {
    state.set_current_conversation(None);
    io.print_conversations(state.list_conversations());
}

fn join(conv: &str, state: &State, io: &IOHandler) {
    if let Some(id) = state.conv_name_to_id(&conv) {
        io.print_messages(state.set_current_conversation(Some(id)).unwrap());
        io.print_log(&format!("Joined conversation with {}.", state.get_current_conversation().unwrap().get_partner().handle));
    } else {
        io.print_error("invalid conversation id");
    }
}

fn list(state: &State, io: &IOHandler) {
    io.print_conversations(state.list_conversations());
}
