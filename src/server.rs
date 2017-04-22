use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;
use std::mem;
use std::io::{self, Read, Write};
use std::str;

extern crate rustc_serialize;
use rustc_serialize::json;

const SERVER_ADDR: &'static str = "0.0.0.0:5001";

#[derive(Clone, RustcEncodable, RustcDecodable, Hash, PartialEq, Eq)]
pub struct UserInfo {
    pub route: Vec<String>,
    pub public_key: [u8; 32],
}

#[derive(Clone, RustcEncodable, RustcDecodable, Hash, PartialEq, Eq)]
pub struct KnownUser {
    pub handle: String,
    pub addr: String,
    pub password: String,
    pub public_key: [u8; 32],
}
type UserMap = Arc<Mutex<HashMap<String, KnownUser>>>;

#[derive(Clone, RustcEncodable, RustcDecodable, Hash, PartialEq, Eq)]
pub struct User {
    pub handle: String,
    pub route: Vec<String>,
    pub public_key: [u8; 32],
}

#[derive(Clone, RustcEncodable, RustcDecodable, PartialEq)]
pub enum ResponseType {
    User (User),
    Connection (UserInfo),
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
        public_key: [u8; 32],
    },
    Connect (String),
    Response (ResponseType)
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Message {
    msg_type: MessageType,
    route: Vec<String>,
}

impl Message {
    pub fn new(msg_type: MessageType, route: Vec<String>) -> Message {
        Message {
            msg_type: msg_type,
            route: route,
        }
    }
}

fn main() {
    let users: UserMap = Arc::new(Mutex::new(HashMap::new()));
    let server = TcpListener::bind(SERVER_ADDR).unwrap();
    
    for stream in server.incoming() {
        if let Ok(stream) = stream {
            let users = users.clone();
            thread::spawn(move || {
                handler(stream, users);
            });
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
    json::decode(s).unwrap()
}


fn send_response(mut stream: TcpStream, res: Message) {

    // Encode the message.
    let encoded_msg: String = json::encode(&res).unwrap();

    // Send the message's size.
    if encoded_msg.len() >= u32::max_value() as usize { return; }
    let msg_size: [u8; 4] = unsafe {
        mem::transmute(encoded_msg.as_bytes().len() as u32)
    };
    stream.write(&msg_size).unwrap();

    // Send the message.
    stream.write(encoded_msg.as_bytes()).unwrap();
}

fn addr_to_string(stream: &TcpStream) -> String {
    match stream.peer_addr().unwrap() {
        SocketAddr::V4(v) => {
            let o = v.ip().octets();
            format!("{}.{}.{}.{}:5000", o[0], o[1], o[2], o[3])
        },
        SocketAddr::V6(v) => {
            let s = v.ip().segments();
            format!("{}.{}.{}.{}.{}.{}.{}.{}:5000", s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7])
        }
    }
}

fn gen_route(user_ip: &str) -> Vec<String> {
    vec![user_ip.to_string()]
}

fn create_response(msg: &Message, users: &UserMap, stream: &TcpStream) -> Message {
    let usr_ip = addr_to_string(&stream);
    let route = gen_route(&usr_ip);
    match msg.msg_type {
        MessageType::Login{ref username, ref password} => {
            match users.lock().unwrap().get(username) {
                Some(u) => {
                    if *password == u.password {
                        Message::new(
                            MessageType::Response(
                                ResponseType::User ( 
                                    User {
                                        handle: u.handle.clone(),
                                        route: vec![usr_ip],
                                        public_key: u.public_key.clone(),
                                    }
                                )
                            ),
                            route
                        )
                    } else {
                        Message::new(
                            MessageType::Response(ResponseType::Error (
                                "Incorrect password.".to_string()
                            )),
                            route
                        )
                    }
                },
                None => {
                    Message::new(
                        MessageType::Response(ResponseType::Error (
                            "User does not exist.".to_string()
                        )),
                        route
                    )
                }
            }
        },
        MessageType::Register{ref username, ref password, ref public_key} => {
            let ref mut users = *users.lock().unwrap();
            // this can probably be simplified using users.entry()
            match users.get(username) {
                Some(_) => Message::new(
                    MessageType::Response(ResponseType::Error (
                        "Username already in use.".to_string()
                    )),
                    route
                ),
                None => {
                    users.insert(username.clone(), KnownUser {
                        handle: username.clone(),
                        addr: usr_ip.clone(),
                        password: password.clone(),
                        public_key: public_key.clone()
                    });
                    Message::new(
                        MessageType::Response(
                            ResponseType::User(
                                User {
                                    handle: username.clone(),
                                    route: vec![usr_ip],
                                    public_key: public_key.clone()
                                }
                            )
                        ), 
                        route
                    )
                }
            }
        },
        MessageType::Connect(ref name) => {
            let ref users = *users.lock().unwrap();
            match users.get(name) {
                Some(known_user) => Message::new(
                    MessageType::Response(
                        ResponseType::Connection(
                            UserInfo {
                                route: vec![known_user.addr.clone()],
                                public_key: known_user.public_key.clone()
                            }
                        )
                    ),
                    route
                ),
                None => Message::new(
                    MessageType::Response(
                        ResponseType::Error(format!("Could not find user {}.", name))
                    ),
                    route
                )
            }
        },
        _ => Message::new(
            MessageType::Response(ResponseType::Error (
                "Command not recognized".to_string()
            )),
            route
        )
    }
}


fn handler(mut stream: TcpStream, users: UserMap) {
    let msg: Message = receive_message(&mut stream);
    let response: Message = create_response(&msg, &users, &stream);
    send_response(stream, response);
}


