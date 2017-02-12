#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_mut)]

//extern crate rustc_serialize;

use std::net::{UdpSocket, SocketAddr};
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel};

mod io_lib;
// mod net_lib;

static SERVER_ADDR: &'static str = "159.203.57.173:5000";
static RECV_ADDR: &'static str = "127.0.0.1:5000";
static SEND_ADDR: &'static str = "127.0.0.1:5001";

fn main() {

    let io_handle = io_lib::IOHandler::new();
   
    // to sender
    let (recv_handler_to_sender, sender_receive) = channel::<String>();
    let input_handler_to_sender = recv_handler_to_sender.clone();

    // to output
    let (recv_handler_to_output, output_receive) = channel::<String>();
    let input_handler_to_output = recv_handler_to_output.clone();

    thread::Builder::new().name("output".to_string()).spawn(move || {

    }).unwrap();

    thread::Builder::new().name("input".to_string()).spawn(move || {
        let (input_done, input_recv) = channel::<i32>();

        loop {
            // move these three into a stuct for easy passing.
            let senders = io_lib::InputHandlerSenders {
                done   : input_done.clone(),
                output : input_handler_to_output.clone(),
                sender : input_handler_to_sender.clone()
            };
            
            let mut input = String::new();
            io_handle.read_prompted_line(&mut input, "> ");

            thread::spawn(move || {
                handle_user_input(&input, &senders);
            });

            // blocks until the handler thread is done taking additional user input.
            let _ = input_recv.recv().unwrap();
        }
    }).unwrap();

    thread::Builder::new().name("receiver".to_string()).spawn(move || {
        let mut socket = UdpSocket::bind(RECV_ADDR)
            .expect("Couldn't bind to socket!");
   
        loop {
            let mut buffer = [0; 4096];
            let (amt, src) = socket.recv_from(&mut buffer)
                .expect("Didn't receive data.");

            thread::spawn(move || {
                receive_handler(&src, &mut buffer[..amt]);
            });
        }
    }).unwrap();

    thread::Builder::new().name("sender".to_string()).spawn(move || {
        let mut socket = Arc::new(Mutex::new(UdpSocket::bind(SEND_ADDR)
            .expect("Couldn't bind to socket!")));

        // how do get thread IDs to then send ACKs back to these threads?
        loop {
            let data = sender_receive.recv();
            thread::spawn(move || {
                
            });
        }
    }).unwrap();

}

fn receive_handler(src: &SocketAddr, buf: &mut [u8]) {

}

fn handle_user_input(data: &String, senders: &io_lib::InputHandlerSenders) {
    println!("user entered: {}", data);
    senders.done.send(0).unwrap();
}

