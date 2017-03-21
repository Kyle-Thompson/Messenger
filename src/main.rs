#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_imports)]

extern crate rustc_serialize;
extern crate crossbeam;

use std::thread;
use std::sync::{Arc, Mutex, Condvar};

mod io_lib;
mod net_lib;
mod mpmc_queue;
mod state;

use net_lib::Net;
use net_lib::TextMessage;
use io_lib::IOHandler;
use state::State;

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
    let is_command = |s: &str| {
        s.chars().nth(0).unwrap() == '/'
    };

    let default_prompt: String = String::from("> ");
    let mut line: String = String::from("");
    
    loop {
        io.read_prompted_line(&mut line, &default_prompt);

        if is_command(&line) {
            // Handle the command.
        } else {
            // Send the message off to the network.
            
            // Print the user's message to the chat.
        }
    }
}

