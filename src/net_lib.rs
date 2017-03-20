use std::net::{UdpSocket, TcpListener, TcpStream, Ipv4Addr, SocketAddr};
use std::thread::{self, JoinHandle};
use std::collections::{VecDeque};
use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;
use std::io::{Read, Write};
use std::str;
use std::mem;
use std::fmt;

use rustc_serialize::json;

use mpmc_queue::MpmcQueue;
use state::User;

const SERVER_ADDR: &'static str = "159.203.57.173:5000";

#[derive(Clone, RustcEncodable, RustcDecodable, PartialEq)]
pub struct TextMessage {
    pub text: String,
    pub sender: User,
    pub conv_id: String,
}

impl fmt::Display for TextMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.sender.handle, self.text)
    }
}

#[derive(RustcEncodable, RustcDecodable, PartialEq)]
enum MessageType {
    Authenticate {
        username: String,
        password: String,
    },
    GetUser {
        handle: String,
    },
    Text {
        msg: TextMessage,
    },
    // File
}

#[derive(RustcEncodable, RustcDecodable)]
struct Message {
    msg_type: MessageType,
    route: Vec<String>,
    // signature
}

struct MessageContainer {
    msg: Message,
    callback: Option<Sender<Message>>,
}

#[derive(Clone)]
pub struct Net {
    send_work: Arc<MpmcQueue<MessageContainer>>,
    recv_work: Arc<MpmcQueue<TcpStream>>,
    new_messages: Arc<MpmcQueue<TextMessage>>,
}

impl Net {

    pub fn new() -> Net {

        // The net struct to be returned.
        let mut net = Net {
            send_work: Arc::new(MpmcQueue::new()),
            recv_work: Arc::new(MpmcQueue::new()),
            new_messages: Arc::new(MpmcQueue::new()),
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
            thread::spawn(move|| { Net::receiver(recv_net); });
        }

        net
    }

    fn main_receiver(net: Net) {
        let server = TcpListener::bind("127.0.0.1:5000").unwrap();

        for stream in server.incoming() {
            match stream {
                Ok(stream) => net.recv_work.push(stream),
                Err(e) => continue,
            }
        }
    }

    fn receiver(net: Net) {

        loop {
            // Grab the connection stream to handle.
            let message: Message = Net::receive_message(&mut net.recv_work.pop());
            
            // Handle the message.
            if message.route.is_empty() { // This message is for us.
                match message.msg_type {
                    MessageType::Text{msg} => net.new_messages.push(msg),
                    _ => continue, // Can't be anything other than text yet.
                }
            } else { // Forward the message along.
                net.send_work.push(MessageContainer{msg: message, callback: None});
            }
        }
    }

    // TODO: Just to be safe, should this not maybe be an optional Message or maybe result?
    fn receive_message(stream: &mut TcpStream) -> Message {

        // Read the message size.
        let mut size_buf: [u8; 4] = [0; 4]; // 32 bit message size field.
        stream.read_exact(&mut size_buf).unwrap();
        let msg_size: u32 = unsafe { mem::transmute(size_buf) };

        // Read the raw message bytes.
        let mut msg_buf: Vec<u8> = Vec::with_capacity(msg_size as usize);
        stream.read_exact(msg_buf.as_mut_slice()).unwrap();

        // Create the message from the raw bytes.
        json::decode(str::from_utf8(&msg_buf).unwrap()).unwrap()
    }

    fn sender(net: Net) {
        let mut element: Option<MessageContainer> = None;

        loop {
            // Grab message from queue.
            let MessageContainer{mut msg, callback} = net.send_work.pop(); 

            // Send the message.
            // TODO: Do something with the error.
            if let Err(e) = Net::send_message(&mut msg) { continue; } 

            // TODO: Implement logic for accepting response if there will be one.
            match msg.msg_type {
                // TODO: find way to match on authenticate without needing to write out
                //       the unnecessary username and password field. All we care about
                //       here is that msg_type is of type Authenticate, we don't actually
                //       use it.
                MessageType::Authenticate{username, password} => continue,
                _ => continue,
            };
        }
    }

    fn send_message(msg: &mut Message) -> Result<(), &'static str> {
    
        // Connect to the destination.
        let dest = msg.route.pop().unwrap();
        let mut stream = TcpStream::connect(dest.as_str()).unwrap();

        // Encode the message.
        let encoded_msg: String = json::encode(msg).unwrap();

        // Send the message's size.
        if encoded_msg.len() >= u32::max_value() as usize {
            return Err("Message is too long."); 
        }
        let msg_size: [u8; 4] = unsafe {
            mem::transmute(encoded_msg.as_bytes().len() as u32)
        };
        stream.write(&msg_size).unwrap();

        // Send the message.
        stream.write(encoded_msg.as_bytes()).unwrap();

        Ok(())
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

    pub fn get_new_message(&self) -> TextMessage {
        self.new_messages.pop()
    }

}
