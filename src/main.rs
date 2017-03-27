#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_imports)]

extern crate rustc_serialize;
extern crate crossbeam;
extern crate rand;

use std::thread;
use std::sync::{Arc, Mutex, Condvar};

mod io_lib;
mod net_lib;
mod mpmc_queue;
mod state;
mod commands;

use net_lib::Net;
use net_lib::TextMessage;
use io_lib::IOHandler;
use state::State;
use state::User;
use commands::*;


fn main() {

    let net = Net::new();
    let io = IOHandler::new();
    let state = State::new();

    crossbeam::scope(|scope| {
        scope.spawn(|| {
            network_receiver(&net, &state);
        });
    });

    crossbeam::scope(|scope| {
        scope.spawn(|| {
            display_output(&io, &state);
        });
    });
    
    crossbeam::scope(|scope| {
        scope.spawn(|| {
            handle_user_input(&io, &net, &state);
        });
    });
}

// Gets a TextMessage from the network and adds it to the new_messages queue in state.
fn network_receiver(net: &Net, state: &State) {
    loop {
        state.add_new_message(net.get_new_message());
    }
}

fn display_output(io: &IOHandler, state: &State) {
    for msg in state.get_new_messages() {
        io.print_new_message(msg);
    }
}

fn handle_user_input(io: &IOHandler, net: &Net, state: &State) {
    let mut user: Option<User> = None;
    let is_command = |s: &str| {
        s.chars().nth(0).unwrap() == '/'
    };

    let mut handle_command = |user: &mut Option<User>, cmd: &str, args: Vec<&str>| {
        match cmd {
            "/login" => {
                *user = match cmd_login(&args, &io, &net) {
                    Ok(usr) => Some(usr),
                    Err(e) => {
                        io.print_error(e);
                        None
                    },
                };
            },
            "/register" => {
                *user = match cmd_register(&args, &io, &net) {
                    Ok(usr) => Some(usr),
                    Err(e) => {
                        io.print_error(e);
                        None
                    },
                };
            },
            "/connect" => {
                if let Err(e) = cmd_connect(&args, &net, &state) {
                    io.print_error(e);
                }
            },

            _ => {
                io.print_error("Command not recognized");
            },
        }
    };
    
    loop {
        let mut line: String = String::from("");
        io.read_prompted_line(&mut line, "> ");

        if is_command(&line) {
            let mut iter = line.split_terminator(' ');
            handle_command(&mut user, iter.next().unwrap(), iter.collect());

        } else {
            let curr_conv = state.get_current_conversation();

            if curr_conv.is_none() {
            
            } else if user.is_none() {

            } else {
                let tm = TextMessage {
                    text: line,
                    sender: user.clone().unwrap(),
                    conv_id: curr_conv.as_ref().unwrap().get_id(),
                };
                
                // Send the message off to the network.
                net.send_text_message(&tm, &curr_conv.unwrap());
                
                // Print the user's message to the chat.
                state.add_new_message(tm);
            }
        }
    }
}

