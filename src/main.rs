#![allow(unused_variables)]
#![allow(unused_mut)]

extern crate rustc_serialize;
extern crate crossbeam;
extern crate rand;

mod io_lib;
mod net_lib;
mod mpmc_queue;
mod state;
mod command;

use net_lib::Net;
use net_lib::Message;
use net_lib::MessageType;
use net_lib::MessageContainer;
use net_lib::TextMessage;
use io_lib::IOHandler;
use state::State;
use state::User;

fn main() {
    let net = Net::new();
    let io = IOHandler::new();
    let state = State::new();
        
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
        let mut line: String = String::from("");
        io.read_prompted_line(&mut line, "> ");

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

                let mc = MessageContainer::new(
                    Message::new(
                        MessageType::Text{msg: tm.clone()}, 
                        net.get_route(&curr_conv.as_ref().unwrap().get_partner().handle).unwrap()
                    ),
                    None
                );
                
                // Send the message off to the network.
                net.add_message(mc);
                
                // Print the user's message to the chat.
                state.add_new_message(tm);
            }
        }
    }
}

