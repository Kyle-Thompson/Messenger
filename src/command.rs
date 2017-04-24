use std::sync::mpsc::channel;
use std::error::Error;
use std::fs::File;
use std::env;
use std::io::Read;

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

    let mut username = io.read_prompted_line("Username: ");
    let mut password = io.read_prompted_line("Password: ");

    // Get the public key.
    let mut public_key = [0u8; 32];
    let mut pub_key_file = File::open(env::home_dir().unwrap().join(".secmsg/keys/public")).unwrap();
    pub_key_file.read_exact(&mut public_key).unwrap();

    let (sender, receiver) = channel();

    net.add_message(
        MessageContainer::new(
            Message::new(
                MessageType::Server(
                    ToServer::Login(username, password, public_key)
                ),
                vec![(Net::server_addr().to_string(), net.get_server_key())],
                &net.crypto
            ),
            Some(sender),
            true
        )
    );

    let res = match receiver.recv().unwrap() {
        Ok(res) => res,
        Err(e) => return Err(e.to_string()),
    };

    if let MessageType::User(res) = Net::data_to_type(&res.unwrap().data) {
        if let ToUser::ServerResponse(res) = res {
            match res {
                ResponseType::User(u) => Ok(u),
                ResponseType::Error(e) => Err(e),
                _ => Err("Something went wrong".to_string())
            }
        } else {
            Err("Reply was not of type ServerResponse".to_string())
        }
    } else {
        Err("Reply was not of type User".to_string())
    }

    // if let MessageType::Response(res) = res.unwrap().msg_type {
    //     match res {
    //         ResponseType::User(u) => Ok(u),
    //         ResponseType::Error(e) => Err(e),
    //         _ => Err("Something went wrong".to_string())
    //     }
    // } else {
    //     Err("Reply was not of type 'Response'. Whut?".to_string())
    // }
}

fn register(io: &IOHandler, net: &Net) -> Result<User, String> {

    let mut username = io.read_prompted_line("Username: ");    
    let mut password = io.read_prompted_line("Password: ");

    // Get the public key.
    let mut public_key = [0u8; 32];
    let mut pub_key_file = File::open(env::home_dir().unwrap().join(".secmsg/keys/public")).unwrap();
    pub_key_file.read_exact(&mut public_key).unwrap();

    let (sender, receiver) = channel();

    net.add_message(
        MessageContainer::new(
            Message::new(
                MessageType::Server(
                    ToServer::Register(username, password, public_key)
                ),
                vec![(Net::server_addr().to_string(), net.get_server_key())],
                &net.crypto
            ),
            Some(sender),
            true
        )
    );

    let res = match receiver.recv() {
        Ok(res) => match res {
            Ok(res) => res,
            Err(e) => return Err(e.to_string()),
        },
        Err(e) => return Err("wtf".to_string() + e.description())
    };

    if let MessageType::User(res) = Net::data_to_type(&res.unwrap().data) {
        if let ToUser::ServerResponse(res) = res {
            match res {
                ResponseType::User(u) => Ok(u),
                ResponseType::Error(e) => Err(e),
                _ => Err("Something went wrong".to_string())
            }
        } else {
            Err("Reply was not of type ServerResponse".to_string())
        }
    } else {
        Err("Reply was not of type User".to_string())
    }
    // if let MessageType::Response(res) = res.unwrap().msg_type {
    //     match res {
    //         ResponseType::User(u) => Ok(u),
    //         ResponseType::Error(e) => Err(e),
    //         _ => Err("Something went wrong".to_string())
    //     }
    // } else {
    //     Err("Reply was not of type 'Response'. Whut?".to_string())
    // }
}

fn connect(o_user: &str, net: &Net, state: &State) -> Result<(), String> {
    let ui: UserInfo = match state.get_user_info(&o_user, net) {
        Ok(ui) => ui,
        Err(e) => return Err(e),
    };

    let conv = Conversation::new(
        User {
            handle: o_user.to_string(),
            addr: ui.addr,
            public_key: ui.public_key
        }
    );
    
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
    } else {
        io.print_error("invalid conversation id");
    }
}

fn list(state: &State, io: &IOHandler) {
    io.print_conversations(state.list_conversations());
}
