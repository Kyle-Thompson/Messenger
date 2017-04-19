use std::net::{TcpListener, TcpStream};
use std::thread::{self};
use std::collections::{HashMap};
use std::collections::hash_map::Entry;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::io::{Read, Write};
use std::str;
use std::mem;
use std::io;

use rustc_serialize::json;

use mpmc_queue::MpmcQueue;
use state::User;

const SERVER_ADDR: &'static str = "138.197.153.113:5001";

#[derive(Clone, RustcEncodable, RustcDecodable, PartialEq)]
pub struct TextMessage {
    pub text: String,
    pub sender: User,
    pub conv_id: u64,
}

impl ToString for TextMessage {
    fn to_string(&self) -> String {
        format!("{}: {}", self.sender.handle, self.text)
    }
}

#[derive(Clone, RustcEncodable, RustcDecodable, PartialEq)]
pub enum ResponseType {
    User (User),
    Connection (Vec<String>),
    Error (String)
}

#[derive(Clone, RustcEncodable, RustcDecodable, PartialEq)]
pub enum MessageType {
    Login {
        username: String,
        password: String,
    },
    Register {
        username: String,
        password: String,
    },
    Connect (String),
    Response (ResponseType),
    Text {
        msg: TextMessage,
    },
    // File
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Message {
    pub msg_type: MessageType,
    route: Vec<String>,
    // signature
}

impl Message {
    pub fn new(msg_type: MessageType, route: Vec<String>) -> Message {
        Message {
            msg_type: msg_type,
            route: route,
        }
    }
}

type Response = Sender<Result<Option<Message>, String>>;

#[derive(Clone)]
pub struct MessageContainer {
    msg: Message,
    response: Option<Response>,
}

impl MessageContainer {
    pub fn new(msg: Message, response: Option<Response>) -> MessageContainer {
        MessageContainer {
            msg: msg,
            response: response,
        }
    }
}

#[derive(Clone)]
pub struct Net {
    send_work: Arc<MpmcQueue<MessageContainer>>,
    recv_work: Arc<MpmcQueue<TcpStream>>,
    new_messages: Arc<MpmcQueue<TextMessage>>,
    routes: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl Net {

    pub fn new() -> Net {

        // The net struct to be returned.
        let mut net = Net {
            send_work: Arc::new(MpmcQueue::new()),
            recv_work: Arc::new(MpmcQueue::new()),
            new_messages: Arc::new(MpmcQueue::new()),
            routes: Arc::new(Mutex::new(HashMap::new())),
        };
       
        // Spawn main receiver.
        let recv_net = net.clone();
        thread::spawn(move|| { Net::listener(recv_net); });

        // Spawning all receiver threads.
        for _ in 0..4 {
            let recv_net = net.clone();
            thread::spawn(move|| { Net::receiver(recv_net); });
        }
        
        // Spawning all sender threads.
        for _ in 0..4 {
            let send_net = net.clone();
            thread::spawn(move|| { Net::sender(send_net); });
        }

        net
    }

    fn listener(net: Net) {
        let server = TcpListener::bind("0.0.0.0:5000").unwrap();

        for stream in server.incoming() {
            match stream {
                Ok(stream) => net.recv_work.push(stream),
                Err(_) => continue,
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
                net.send_work.push(MessageContainer{msg: message, response: None});
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
        let mut msg_buf = vec![0; msg_size as usize];
        stream.read_exact(msg_buf.as_mut_slice()).unwrap();

        // Create the message from the raw bytes.
        let s = str::from_utf8(&msg_buf).unwrap();
        //println!("Got: {}", s);
        //io::stdout().flush().unwrap();
        json::decode(s).unwrap()
    }

    fn sender(net: Net) {

        loop {
            // Grab message from queue.
            let MessageContainer{mut msg, response} = net.send_work.pop(); 
            
            // Connect to the destination.
            let dest: String = msg.route.pop().unwrap();
            let mut stream: TcpStream = match TcpStream::connect(&*dest) {
                Ok(s) => s,
                Err(_) => {
                    if let Some(res) = response {
                        res.send(Err("Could not connect to destination".to_string())).unwrap();
                        // TODO: Let server know about the disconnected host.
                    }
                    continue;
                }
            };

            // Send the message.
            // TODO: Do something with the error.
            if let Err(_) = Net::send_message(&mut stream, &mut msg) { 
                println!("Error sending message");
                io::stdout().flush().unwrap();
                continue; 
            } 

            // Get the response message if there will be one.
            if let Some(res) = response {
                if Net::needs_response(&msg.msg_type) {
                    res.send(Ok(Some(Net::receive_message(&mut stream)))).unwrap();
                } else {
                    res.send(Ok(None)).unwrap();
                }
            }
        }
    }

    fn send_message(stream: &mut TcpStream, msg: &mut Message) -> Result<(), &'static str> {

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

    fn needs_response(msg_type: &MessageType) -> bool {
        match *msg_type {
            MessageType::Login{ref username, ref password} => true,
            MessageType::Register{ref username, ref password} => true,
            MessageType::Connect(ref user) => true,
            _ => false,
        }
    }
    
    pub fn get_message(&self) -> TextMessage {
        self.new_messages.pop()
    }

    pub fn add_message(&self, msg: MessageContainer) {
        self.send_work.push(msg);
    }

    pub fn get_route(&self, name: &str) -> Result<Vec<String>, String> {
        match self.routes.lock().unwrap().entry(name.to_string()) {
            Entry::Occupied(o) => Ok(o.get().clone()),
            Entry::Vacant(v) => {
                match self.gen_route(name) {
                    Ok(r) => {
                        Ok(v.insert(r).clone())
                    },
                    Err(e) => Err(e.to_string())
                }
            }
        }
    }

    fn gen_route(&self, user: &str) -> Result<Vec<String>, String> {
        let (sender, receiver) = channel();
        self.add_message(
            MessageContainer::new(
                Message::new(
                    MessageType::Connect(user.to_string()),
                    vec![SERVER_ADDR.to_string()]
                ),
                Some(sender)
            )
        );

        let res = match receiver.recv().unwrap(){
            Ok(r) => r.unwrap(),
            Err(e) => {
                return Err(e);
            }
        };

        if let MessageType::Response(res) = res.msg_type {
            match res {
                ResponseType::Connection(route) => Ok(route),
                ResponseType::Error(e) => Err(e),
                _ => Err("Something went wrong".to_string())
            }
        } else {
            Err("Reply was not of type 'Response'. Whut?".to_string())
        }
    }

    pub fn server_addr() -> &'static str {
        SERVER_ADDR
    }

}
