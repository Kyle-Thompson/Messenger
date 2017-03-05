#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_imports)]

extern crate rustc_serialize;

use std::net::{UdpSocket, SocketAddr};
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};

mod io_lib;
mod net_lib;
mod mpmc_queue;
mod state;

fn main() {

    let net = net_lib::Net::new();
    let io_handle = io_lib::IOHandler::new();

    /*let mut input_queue: Arc<MpmcQueue<(String, Sender<bool>)>> = Arc::new(MpmcQueue::new());

    for _ in 0..4 {
        let queue = input_queue.clone();
        thread::spawn(move|| { input_handler(queue); });
    }
   
    let input = thread::Builder::new().name("input".to_string()).spawn(move || {
        let (input_done, input_recv) = channel::<bool>();

        loop {
            
            //let mut input = io_handle.get_line();

            let done = input_done.clone();
            thread::spawn(move || {
                //handle_user_input(io_handle.get_line(), done);
            });

            // blocks until the handler thread is done taking additional user input.
            // turn into a match to see if thread should end.
            let _ = input_recv.recv().unwrap();
        }
    }).unwrap();

    //let done = input.join();*/
    
}

fn handle_user_input(data: String) {
    println!("user entered: {}", data); // Temporary.
    //senders.input.send(0).unwrap(); // Release input buffer back to input thread.
}

fn input_handler(queue: Arc<MpmcQueue<(String, Sender<bool>)>>) {
    loop {

    }
}







