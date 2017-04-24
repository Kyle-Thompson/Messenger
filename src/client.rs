#![allow(unused_variables)]
#![allow(unused_mut)]

extern crate rustc_serialize;
extern crate crossbeam;
extern crate rand;
extern crate crypto;

use std::fs::{self, File};
use std::io::{Read, Write};
use std::env;
use std::process;

mod io_lib;
mod net_lib;
mod mpmc_queue;
mod state;
mod command;
mod crypto_lib;

use net_lib::Net;
use net_lib::Message;
use net_lib::MessageType;
use net_lib::MessageContainer;
use net_lib::TextMessage;
use net_lib::ToUser;
use crypto_lib::Crypto;
use io_lib::IOHandler;
use state::State;
use state::User;

fn main() {

    let io = IOHandler::new();
    let state = State::new();

    let (priv_key, pub_key) = {
        let mut keydir = match env::home_dir() {
            Some(p) => p,
            None    => {
                io.print_error("Cannot find home directory.");
                process::exit(1);
            }
        };

        keydir.push(".secmsg/keys");
        if !keydir.join("private").exists() || !keydir.join("public").exists() {
            fs::create_dir_all(&keydir).unwrap();

            let (priv_key, pub_key) = crypto_lib::gen_key_pair();
            
            let mut priv_key_file = File::create(keydir.join("private")).unwrap();
            priv_key_file.write_all(&priv_key).unwrap();

            let mut pub_key_file = File::create(keydir.join("public")).unwrap();
            pub_key_file.write_all(&pub_key).unwrap();

            (priv_key, pub_key)
        } else {
            let mut priv_key = [0u8; 32];
            let mut priv_key_file = File::open(keydir.join("private")).unwrap();
            priv_key_file.read_exact(&mut priv_key).unwrap();

            let mut pub_key = [0u8; 32];
            let mut pub_key_file = File::open(keydir.join("public")).unwrap();
            pub_key_file.read_exact(&mut pub_key).unwrap();

            (priv_key, pub_key)
        }
    };
    let net = Net::new(Crypto::new(priv_key, pub_key));
        
    crossbeam::scope(|scope| {
        scope.spawn(|| {
            network_receiver(&net, &state);
        });
        
        scope.spawn(|| {
            display_output(&io, &state);
        });
        
        handle_user_input(&io, &net, &state);
    });
}

// Gets a TextMessage from the network and adds it to the new_messages queue in state.
fn network_receiver(net: &Net, state: &State) {
    loop {
        state.add_new_message(net.get_message());
    }
}

fn display_output(io: &IOHandler, state: &State) {
    for msg in state.get_new_messages() {
        io.print_message(msg);
    }
}

fn handle_user_input(io: &IOHandler, net: &Net, state: &State) {
    let mut user: Option<User> = None;
    let is_command = |s: &str| {
        s.chars().nth(0).unwrap() == '/'
    };
    
    loop {
        let mut line = io.read_prompted_line("> ");

        if is_command(&line) {
            let tokens: Vec<&str> = line.split_terminator(' ').collect();
            command::handle(&io, &net, &state, &mut user, &*tokens);

        } else {
            let curr_conv = state.get_current_conversation();
            
            if curr_conv.is_none() {
                io.print_error("No current conversation.");
                continue;
            } else if user.is_none() {
                io.print_error("Not logged in");
                continue;
            } else {
                let conv_id = curr_conv.as_ref().unwrap().get_id(); 
                let tm = TextMessage {
                    text: line,
                    sender: user.clone().unwrap(),
                    conv_id: conv_id,
                };
                let partner = curr_conv.as_ref().unwrap().get_partner();
                let mc = MessageContainer::new(
                    Message::new(
                        MessageType::User(ToUser::Text(tm.clone())), 
                        state.get_user_info(&partner.handle, &net).unwrap().route,
                        &net.crypto
                    ),
                    None,
                    false
                );
                
                // Send the message off to the network.
                net.add_message(mc);
                
                // Print the user's message to the chat.
                // state.add_new_message(tm);
            }
        }
    }
}

