#![allow(dead_code)]

use std::sync::mpsc::Sender;

use rustc_serialize::json;

use state::User;
use state::Route;
use crypto_lib::Crypto;
use crypto_lib::Key;

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
    Connection (Route),
    PublicKey (Key),
    Error (String),
}

#[derive(Clone, RustcEncodable, RustcDecodable, PartialEq)]
pub enum ToServer {
    Login (String, String, Key), // username, password, public key
    Register (String, String, Key), // username, password, public key
    Connect (String, Key), // other user's name, public key
    PublicKey (Key), // public key
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

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Message {
    pub data: Vec<u8>,
    pub next_hop: Option<String>,
}

impl Message {
    pub fn new(msg_type: MessageType, route: Route, crypto: &Crypto) -> Message {
        route.into_iter().fold(Message {
            data: json::encode(&msg_type).unwrap().into_bytes(),
            next_hop: None
        }, |m, r| {
            Message {
                data: crypto.encrypt(&r.1, json::encode(&m).unwrap().as_bytes()).unwrap(),
                next_hop: Some(r.0)
            }
        })
    }
}

type Response = Sender<Result<Option<Message>, String>>;

#[derive(Clone)]
pub struct MessageContainer {
    pub msg: Message,
    pub response: Option<Response>,
    pub needs_response: bool,
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