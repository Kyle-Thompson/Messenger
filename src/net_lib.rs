use std::net::{UdpSocket, TcpListener, TcpStream, Ipv4Addr, SocketAddr};
use std::thread::{self, JoinHandle};
use std::collections::{VecDeque};
use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;
use std::io::{Read, Write};
use std::str;
use std::mem;

use rustc_serialize::json;

use mpmc_queue::MpmcQueue as MpmcQueue;
use state::State as State;
use state::User as User;

const SERVER_ADDR: &'static str = "159.203.57.173:5000";

#[derive(RustcEncodable, RustcDecodable, PartialEq)]
enum MessageType { // TODO: fill in later
    Ack,
    Authenticate {
        username: String,
        password: String
    },
}

#[derive(RustcEncodable, RustcDecodable)]
struct Message {
    msg_type: MessageType,
    route: Vec<String>,
    // signature
    // id
}

struct MessageContainer {
    msg: Message,
    callback: Option<Sender<Message>>,
}

#[derive(Clone)]
pub struct Net {
    send_work: Arc<MpmcQueue<MessageContainer>>,
    recv_work: Arc<MpmcQueue<TcpStream>>,
}

impl Net {

    pub fn new() -> Net {

        // The net struct to be returned.
        let mut net = Net {
            send_work: Arc::new(MpmcQueue::new()),
            recv_work: Arc::new(MpmcQueue::new()),
        };
        
        // Spawning all sender threads.
        for i in 0..16 {
            let send_net = net.clone();
            thread::spawn(move|| { Net::sender(send_net); });
        }
       
        // Spawn main receiver.
        let recv_net = net.clone();
        thread::spawn(move|| { Net::main_receiver(recv_net); });

        // Spawning all receiver handler threads.
        for i in 0..16 {
            let recv_net = net.clone();
            thread::spawn(move|| { Net::receive_handler(recv_net); });
        }

        net
    }

    fn main_receiver(net: Net) {
        let server = TcpListener::bind("127.0.0.1:5000").unwrap();

        for stream in server.incoming() {
            match stream {
                Ok(stream) => net.recv_work.push(stream),
                Err(e) => continue, // TODO handle this error
            }
        }
    }

    fn receive_handler(net: Net) {
        let mut size_buf: [u8; 4] = [0; 4]; // 32 bit message size field.

        loop {
            // Grab the connection stream to handle.
            let mut stream = match net.recv_work.pop() {
                Some(s) => s,
                None    => continue,
            };

            // Read the message size.
            stream.read_exact(&mut size_buf).unwrap();
            let msg_size: u32 = unsafe { mem::transmute(size_buf) };

            // Read the raw message bytes.
            let mut msg_buf: Vec<u8> = Vec::with_capacity(msg_size as usize);
            stream.read_exact(msg_buf.as_mut_slice()).unwrap();

            // Create the message from the raw bytes.
            let message: Message = json::decode(str::from_utf8(&msg_buf).unwrap()).unwrap();

            // TODO: Handle message.
        }
    }

    fn sender(net: Net) {
        //let mut stream = TcpStream::bind("127.0.0.1:0").expect("Couldn't bind socket!");
        let mut element: Option<MessageContainer> = None;

        loop {
            // Grab message from queue.
            let (mut msg, callback) = match net.send_work.pop() {
                Some(MessageContainer{msg: m, callback: c}) => (m, c),
                None => continue,
            };

            // Connect to the destination.
            let dest = msg.route.pop().unwrap();
            let mut stream = TcpStream::connect(dest.as_str()).unwrap();

            // Encode the message.
            let encoded_msg: String = json::encode(&msg).unwrap();//.as_bytes();

            // Send the message's size.
            if encoded_msg.len() >= u32::max_value() as usize { continue; }
            let msg_size: [u8; 4] = unsafe {
                mem::transmute(encoded_msg.as_bytes().len() as u32)
            };
            stream.write(&msg_size).unwrap();

            // Send the message.
            stream.write(encoded_msg.as_bytes()).unwrap();
            
/*
            // send message off [deprecated]
            let mut buffer = [0; 4096];
            let dest = msg.route.pop().unwrap();
            'send: loop {
                //socket.send_to(json::encode(&msg).unwrap().as_bytes(), dest.as_str())
                //    .expect("Couldn't send data!");

                let mut resp_msg_size: usize;
                'recv: loop {
                    match socket.recv_from(&mut buffer) {
                        Ok((resp_size, resp_src)) => {
                            // remove when more advanced sender authentication is used
                            if resp_src != dest.parse().unwrap() { continue 'recv; } 
                            resp_msg_size = resp_size;
                            break 'recv;
                        },
                        _  => continue 'send,
                    }
                }
                
                // TODO: add error handling
                let res_msg: Message = json::decode(
                    str::from_utf8(&buffer[..resp_msg_size]).unwrap()).unwrap();

            } */
        }
    }

    /*pub fn authenticate_user(&self, username: String, password: String) {
        let (sender, receiver) = channel::<Message>();
        let &(ref queue, ref cvar) = &*self.work;
        
        {
            let mut queue = queue.lock().unwrap();
            queue.push_back(MessageContainer{
                msg: Message {
                    msg_type: MessageType::Authenticate{
                        username: username, password: password
                    },
                    route: vec![SERVER_ADDR.to_string()],
                },
                callback: Some(sender),
            });
        }
        cvar.notify_one();

        let received = receiver.recv().unwrap();

        // now do stuff with what was received.
    }*/

    /*pub fn send(&self, user: User) {//message: Message) {
        let &(ref queue, ref cvar) = &*self.work;
        {
            let mut queue = queue.lock().unwrap();
            //queue.push_back(MessageContainer{msg: message, callback: None});
        }
        cvar.notify_one();
    }*/

}
