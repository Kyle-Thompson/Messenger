#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_imports)]

extern crate rustc_serialize;

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

    let net = Arc::new(Net::new());
    let io = Arc::new(IOHandler::new());
    let state = Arc::new((Mutex::new(State::new()), Condvar::new()));

    
    { // Network receiver
        let net = net.clone();
        let state = state.clone();
        thread::spawn(move|| { network_receiver(net, state); });
    }

    { // Display output
        let io = io.clone();
        let state = state.clone();
        thread::spawn(move|| { display_output(io, state); });
    }

    { // Get user input
        let io = io.clone();
        let net = net.clone();
        let state = state.clone();
        thread::spawn(move|| {
            handle_user_input(io, net, state);        
        }).join().unwrap(); // Only returns when this thread finishes.
    }
}

// Gets a TextMessage from the network and adds it to the new_messages queue in state.
fn network_receiver(net: Arc<Net>, state: Arc<(Mutex<State>, Condvar)>) {
    loop {
        let msg: TextMessage = net.get_new_message();
    }
}

fn display_output(io: Arc<IOHandler>, state: Arc<(Mutex<State>, Condvar)>) {
    let &(ref mutex, ref cvar) = &*state;
    loop {
        let mut state = mutex.lock().unwrap();
    }
}

fn handle_user_input(io: Arc<IOHandler>, net: Arc<Net>, state: Arc<(Mutex<State>, Condvar)>) {
    let default_prompt: String = String::from("> ");
    let mut line: String = String::from("");
    
    loop {
        io.read_prompted_line(&mut line, &default_prompt);
    }
}


