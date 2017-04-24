#![allow(dead_code)]

use std::net::{TcpListener, TcpStream};
use std::thread::{self};
use std::sync::{Arc};
use std::sync::mpsc::{channel, Sender};
use std::io::{self, Read, Write};
use std::str;
use std::mem;
//use std::io;

use rustc_serialize::json;

use mpmc_queue::MpmcQueue;
use state::User;
use state::UserInfo;
use crypto_lib::Crypto;
use crypto_lib::KeyArr;

const SERVER_ADDR: &'static str = "138.197.153.113:5001";
const SERVER_KEY_ADDR: &'static str = "138.197.153.113:5002";

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
    Connection (UserInfo),
    PublicKey (KeyArr),
    Error (String),
}

#[derive(Clone, RustcEncodable, RustcDecodable, PartialEq)]
pub enum ToServer {
    Login (String, String, KeyArr), // username, password, public key
    Register (String, String, KeyArr), // username, password, public key
    Connect (String, KeyArr), // other user's name, public key
    PublicKey (KeyArr), // public key
}

#[derive(Clone, RustcEncodable, RustcDecodable, PartialEq)]
pub enum ToUser {
    ServerResponse (ResponseType),
    Text (TextMessage),
    // File
}

#[derive(Clone, RustcEncodable, RustcDecodable, PartialEq)]
pub enum MessageType {
    Server(ToServer),
    User(ToUser),
}

// #[derive(Clone, RustcEncodable, RustcDecodable)]
// pub struct Message {
//     pub msg_type: MessageType,
//     route: Vec<String>,
//     // signature
// }

// impl Message {
//     pub fn new(msg_type: MessageType, route: Vec<String>) -> Message {
//         Message {
//             msg_type: msg_type,
//             route: route,
//         }
//     }
// }

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Message {
    pub data: Vec<u8>,
    next_hop: Option<String>,
}

impl Message {
    pub fn new(msg_type: MessageType, route: Vec<(String, KeyArr)>, crypto: &Crypto) -> Message {
        let msg_type2 = json::encode(&msg_type).unwrap().into_bytes();
        let mut msg = Message {
            data: msg_type2,
            next_hop: None,
        };

        // println!("route size {}", route.len());
        // println!("Assembling message:\n{}", json::encode(&msg_type).unwrap());
        // println!("Step: {}", json::encode(&msg).unwrap());

        for (hop, key) in route {
            msg = Message {
                data: crypto.encrypt(&key, json::encode(&msg).unwrap().as_bytes()).unwrap(),
                next_hop: Some(hop),
            };
            //println!("Step: {}", json::encode(&msg).unwrap());
        }
        //io::stdout().flush().unwrap();
        msg
    }
}

type Response = Sender<Result<Option<Message>, String>>;

#[derive(Clone)]
pub struct MessageContainer {
    msg: Message,
    response: Option<Response>,
    needs_response: bool,
    //key: Option<KeyArr>
}

impl MessageContainer {
    pub fn new(msg: Message, res: Option<Response>, need_res: bool) -> MessageContainer {
        MessageContainer {
            msg: msg,
            response: res,
            needs_response: need_res,
        }
    }
}

#[derive(Clone)]
pub struct Net {
    send_work: Arc<MpmcQueue<MessageContainer>>,
    recv_work: Arc<MpmcQueue<TcpStream>>,
    new_messages: Arc<MpmcQueue<TextMessage>>,
    pub crypto: Crypto,
    server_key: KeyArr,
}

impl Net {

    pub fn new(crypto: Crypto) -> Net {

        // Get the server's public key.
        let mut stream: TcpStream = TcpStream::connect(SERVER_KEY_ADDR).unwrap();
        let mut key_request = Message::new(
            MessageType::Server(
                ToServer::PublicKey(crypto.pub_key)
            ),
            vec![],
            &crypto
        );
        Net::send_message(&mut stream, &mut key_request).unwrap();
        let msg_type: MessageType = Net::data_to_type(&Net::receive_message(&mut stream, &crypto).data);

        let server_pub_key = match msg_type {
            MessageType::User(m) => {
                match m {
                    ToUser::ServerResponse(sr) => {
                        match sr {
                            ResponseType::PublicKey(pk) => pk,
                            _ => panic!("Unable to get server public key.")
                        }
                    },
                    _ => panic!("Unable to get server public key.")
                }
            },
            _ => panic!("Unable to get server public key.")
        };

        // The net struct to be returned.
        let net = Net {
            send_work: Arc::new(MpmcQueue::new()),
            recv_work: Arc::new(MpmcQueue::new()),
            new_messages: Arc::new(MpmcQueue::new()),
            crypto: crypto,
            server_key: server_pub_key,
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
            let message: Message = Net::receive_message(&mut net.recv_work.pop(), &net.crypto);
            
            // Handle the message.
            if message.next_hop == None { // This message is for us.
                match Net::data_to_type(&message.data) {
                    MessageType::User(mtu) => match mtu {
                        ToUser::Text(ref msg) => net.new_messages.push(msg.clone()),
                        _ => continue, // Can't be anything other than text yet.
                    },
                    MessageType::Server(_) => continue,
                }
            } else { // Forward the message along.
                println!("Forwarding message");
                io::stdout().flush().unwrap();
                net.send_work.push(MessageContainer{
                    msg: message, 
                    response: None,
                    needs_response: false,
                });
            }
        }
    }

    // // TODO: Just to be safe, should this not maybe be an optional Message or maybe result?
    // fn receive_message(stream: &mut TcpStream) -> Message {

    //     // Read the message size.
    //     let mut size_buf: [u8; 4] = [0; 4]; // 32 bit message size field.
    //     stream.read_exact(&mut size_buf).unwrap();
    //     let msg_size: u32 = unsafe { mem::transmute(size_buf) };

    //     // Read the raw message bytes.
    //     let mut msg_buf = vec![0; msg_size as usize];
    //     stream.read_exact(msg_buf.as_mut_slice()).unwrap();

    //     // Create the message from the raw bytes.
    //     let s = str::from_utf8(&msg_buf).unwrap();
    //     json::decode(s).unwrap()
    // }

    // TODO: Just to be safe, should this not maybe be an optional Message or maybe result?
    fn receive_message(stream: &mut TcpStream, crypto: &Crypto) -> Message {

        // Read the message size.
        let mut size_buf: [u8; 4] = [0; 4]; // 32 bit message size field.
        stream.read_exact(&mut size_buf).unwrap();
        let msg_size: u32 = unsafe { mem::transmute(size_buf) };

        // Read the raw message bytes.
        let mut msg_buf = vec![0; msg_size as usize];
        stream.read_exact(msg_buf.as_mut_slice()).unwrap();

        // Decrypt the message.
        // TODO: this should be a match that can return an error
        let decrypted_message = crypto.decrypt(&msg_buf).unwrap();

        // Create the message from the raw bytes.
        json::decode(str::from_utf8(&decrypted_message).unwrap()).unwrap()
    }

    fn sender(net: Net) {

        loop {
            // Grab message from queue.
            let MessageContainer{mut msg, response, needs_response} = net.send_work.pop(); 
            
            // Connect to the destination.
            //let dest: String = ;
            let mut stream: TcpStream = match TcpStream::connect(&*msg.clone().next_hop.unwrap()) {
                Ok(s) => s,
                Err(_) => {
                    if let Some(res) = response {
                        res.send(Err("Could not connect to destination".to_string())).unwrap();
                    }
                    continue;
                }
            };

            // Generate encryption function.
            // let enc_func = |&m| match key {
            //     Some(key) => net.crypto.encrypt(&key, &m),
            //     None      => Ok(m.to_vec())
            // };

            // Send the message.
            // TODO: Do something with the error.
            if let Err(_) = Net::send_message(&mut stream, &mut msg) { 
                continue; 
            } 

            // Get the response message if there will be one.
            if let Some(res) = response {
                if needs_response {
                    res.send(Ok(Some(Net::receive_message(&mut stream, &net.crypto)))).unwrap();
                } else {
                    res.send(Ok(None)).unwrap();
                }
            }
        }
    }

    fn send_message(stream: &mut TcpStream, msg: &mut Message) -> Result<(), &'static str> {

        // // Encode the message in JSON.
        // let encoded_msg: Vec<u8> = json::encode(msg).unwrap().into_bytes();

        // // Encrypt the message.
        // let encrypted_msg: Vec<u8> = match key {
        //     Some(key) => crypto.encrypt(&key, &encoded_msg).unwrap(),
        //     None      => encoded_msg
        // };

        // Check the message size.
        if msg.data.len() >= u32::max_value() as usize {
            return Err("Message is too long."); 
        }

        // Send the message size.
        let msg_size: [u8; 4] = unsafe { // TODO: should this be encrypted too?
            mem::transmute(msg.data.len() as u32)
        };
        stream.write(&msg_size).unwrap();

        // Send the message.
        stream.write(&msg.data).unwrap();

        Ok(())
    }

    // pub fn unencrypted_data_to_type(data: &[u8]) -> MessageType {
    //     json::decode(str::from_utf8(&data).unwrap()).unwrap()
    // }

    pub fn data_to_type(data: &[u8]) -> MessageType {
        json::decode(str::from_utf8(&data).unwrap()).unwrap()
    }

    fn data_to_message(data: &[u8], crypto: &Crypto) -> Message {
        let decrypted = crypto.decrypt(&data).unwrap();
        json::decode(str::from_utf8(&decrypted).unwrap()).unwrap()
    }

    fn needs_response(msg_type: &MessageType) -> bool {
        match *msg_type {
            MessageType::Server(_) => true,
            MessageType::User(_) => false,
        }
    }

    pub fn get_server_key(&self) -> KeyArr {
        self.server_key.clone()
    }
    
    pub fn get_message(&self) -> TextMessage {
        self.new_messages.pop()
    }

    pub fn add_message(&self, msg: MessageContainer) {
        self.send_work.push(msg);
    }

    pub fn get_user_info(&self, user: &str) -> Result<UserInfo, String> {
        let (sender, receiver) = channel();
        self.add_message(
            MessageContainer::new(
                Message::new(
                    MessageType::Server(
                        ToServer::Connect(user.to_string(), self.crypto.pub_key.clone())
                    ),
                    vec![(SERVER_ADDR.to_string(), self.server_key)],
                    &self.crypto
                ),
                Some(sender),
                true            )
        );

        let res = match receiver.recv().unwrap(){
            Ok(r) => r.unwrap(),
            Err(e) => {
                return Err(e);
            }
        };

        if let MessageType::User(res) = Net::data_to_type(&res.data) {
            if let ToUser::ServerResponse(res) = res {
                match res {
                    ResponseType::Connection(u) => Ok(u),
                    ResponseType::Error(e) => Err(e),
                    _ => Err("Something went wrong".to_string())
                }
            } else {
                Err("Reply was not of type ServerResponse".to_string())
            }
        } else {
            Err("Reply was not of type User".to_string())
        }
    }

    pub fn server_addr() -> &'static str {
        SERVER_ADDR
    }

}
