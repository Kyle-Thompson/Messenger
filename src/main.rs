use std::net::{UdpSocket};
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, channel};

mod io_lib;
// mod net_lib;


//struct User {
//    handle: String,
//    password: String,
    // public_key: String,
    // private_key: String,
//}

fn main() {
   
    // to sender
    let (recv_handler_to_sender, sender_receive) = channel::<String>();
    let input_handler_to_sender = recv_handler_to_sender.clone();

    // to output
    let (recv_handler_to_output, output_receive) = channel::<String>();
    let input_handler_to_output = recv_handler_to_output.clone();

    thread::Builder::new().name("receiver".to_string()).spawn(move || {
        let mut socket = UdpSocket::bind("127.0.0.1:5000")
            .expect("Couldn't bind to socket!");
    
        loop {
            
        }
    });

    thread::Builder::new().name("sender".to_string()).spawn(move || {
        let mut socket = Arc::new(Mutex::new(UdpSocket::bind("127.0.0.1:5001")
            .expect("Couldn't bind to socket!")));

        // how do get thread IDs to then send ACKs back to these threads.
        loop {
            let data = sender_receive.recv();
            thread::spawn(move || {
                
            });
        }
    });
    
    thread::Builder::new().name("output".to_string()).spawn(move || {

    });
    
    login();
}

fn login() {
    
    let mut username = String::new();
    let mut password = String::new();

    loop {
        io_lib::read_prompted_line(&mut username, "username: ");
        io_lib::read_prompted_line(&mut password, "password: ");

        // TODO: add authentication check.
        break;
    }

    //User { 
    //    handle: username.trim().to_string(), 
    //    password: password.trim().to_string()
    //}
}
