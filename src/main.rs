#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_imports)]

extern crate rustc_serialize;

use std::net::{UdpSocket, SocketAddr};
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel};

mod io_lib;
mod net_lib;

static RECV_ADDR: &'static str = "127.0.0.1:5000";

fn main() {

    let net = net_lib::Net::new();
    let io_handle = io_lib::IOHandler::new();
   
    // to sender
    let (recv_handler_to_sender, sender_receive) = channel::<String>();
    let input_handler_to_sender = recv_handler_to_sender.clone();

    // to output
    let (recv_handler_to_output, output_receive) = channel::<String>();
    let input_handler_to_output = recv_handler_to_output.clone();

    thread::Builder::new().name("output".to_string()).spawn(move || {

    }).unwrap();

    let input = thread::Builder::new().name("input".to_string()).spawn(move || {
        let (input_done, input_recv) = channel::<i32>();

        loop {
            // move these three into a stuct for easy passing.
            let senders = io_lib::InputHandlerSenders {
                input  : input_done.clone(),
                output : input_handler_to_output.clone(),
                sender : input_handler_to_sender.clone()
            };
            
            let mut input = String::new();
            io_handle.read_prompted_line(&mut input, "> ");

            thread::spawn(move || {
                handle_user_input(&input, &senders);
            });

            // blocks until the handler thread is done taking additional user input.
            // turn into a match to see if thread should end.
            let _ = input_recv.recv().unwrap();
        }
    }).unwrap();

    let done = input.join();

}

fn handle_user_input(data: &String, senders: &io_lib::InputHandlerSenders) {
    println!("user entered: {}", data); // Temporary.
    senders.input.send(0).unwrap(); // Release input buffer back to input thread.
}









