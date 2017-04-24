use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;
use std::mem;
use std::io::{Read, Write};
use std::str;
use std::cmp;
use std::env;
use std::fs::{self, File};
//use std::io;

extern crate rustc_serialize;
use rustc_serialize::json;
extern crate crossbeam;
extern crate crypto;
extern crate rand;

mod io_lib;
mod net_lib;
mod mpmc_queue;
mod state;
mod crypto_lib;

use net_lib::{Message, MessageType, ResponseType};
use net_lib::{ToUser, ToServer};
use net_lib::Net;
use crypto_lib::Crypto;
use crypto_lib::KeyArr;
use state::User;
use state::UserInfo;

const SERVER_ADDR: &'static str = "0.0.0.0:5001";
const PUB_KEY_ADDR: &'static str = "0.0.0.0:5002";

#[derive(Clone, RustcEncodable, RustcDecodable, Hash, PartialEq, Eq)]
pub struct KnownUser {
    pub handle: String,
    pub addr: String,
    pub password: String,
    pub public_key: [u8; 32],
}
type UserMap = Arc<Mutex<HashMap<String, KnownUser>>>;

fn main() {
    let (priv_key, pub_key) = {
        let mut keydir = env::home_dir().unwrap();

        keydir.push(".secmsg/keys");
        if !keydir.join("private").exists() || !keydir.join("public").exists() {
            fs::create_dir_all(&keydir).unwrap();

            let (priv_key, pub_key) = crypto_lib::gen_key_pair();
            
            let mut priv_key_file = File::create(keydir.join("private")).unwrap();
            priv_key_file.write_all(&priv_key).unwrap();

            let mut pub_key_file = File::create(keydir.join("public")).unwrap();
            pub_key_file.write_all(&pub_key).unwrap();

            (priv_key, pub_key)
        } else {
            let mut priv_key = [0u8; 32];
            let mut priv_key_file = File::open(keydir.join("private")).unwrap();
            priv_key_file.read_exact(&mut priv_key).unwrap();

            let mut pub_key = [0u8; 32];
            let mut pub_key_file = File::open(keydir.join("public")).unwrap();
            pub_key_file.read_exact(&mut pub_key).unwrap();

            (priv_key, pub_key)
        }
    };
    let crypto = Crypto::new(priv_key, pub_key);

    let users: UserMap = Arc::new(Mutex::new(HashMap::new()));
    let server = TcpListener::bind(SERVER_ADDR).unwrap();
    
    crossbeam::scope(|scope| {
        scope.spawn(|| {
            for stream in server.incoming() {
                if let Ok(stream) = stream {
                    let users = users.clone();
                    let crypto = crypto.clone(); // TODO: Can this be avoided?
                    thread::spawn(move || {
                        handler(stream, users, crypto);
                    });
                }
            }
        });

        scope.spawn(|| {
            for stream in TcpListener::bind(PUB_KEY_ADDR).unwrap().incoming() {
                if let Ok(stream) = stream {
                    pub_key_handler(stream, pub_key.clone(), &crypto);
                }
            }
        });
    });
}


// TODO: Just to be safe, should this not maybe be an optional Message or maybe result?
fn receive_unencrypted_message_type(stream: &mut TcpStream) -> MessageType {

    // Read the message size.
    let mut size_buf: [u8; 4] = [0; 4]; // 32 bit message size field.
    stream.read_exact(&mut size_buf).unwrap();
    let msg_size: u32 = unsafe { mem::transmute(size_buf) };

    // Read the raw message bytes.
    let mut msg_buf = vec![0; msg_size as usize];
    stream.read_exact(msg_buf.as_mut_slice()).unwrap();

    // Create the message from the raw bytes.
    json::decode(str::from_utf8(&msg_buf).unwrap()).unwrap()
}

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


fn send_response(mut stream: TcpStream, res: Message) {

    // Encode the message.
    //println!("Message before encoding: {}", json::encode(&res).unwrap());
    //let encoded_msg: Vec<u8> = json::encode(&res.data).unwrap().into_bytes();

    //println!("Number of bytes before encrypt {}", encoded_msg.len());

    // Encrypt the message.
    //let encrypted_msg: Vec<u8> = crypto.encrypt(&key, &encoded_msg).unwrap();
    //println!("Number of bytes after encrypt {}", encrypted_msg.len());
    //io::stdout().flush().unwrap();

    // Check the message size.
    if res.data.len() >= u32::max_value() as usize { return; }

    // Send the message size.
    let msg_size: [u8; 4] = unsafe {
        mem::transmute(res.data.len() as u32)
    };
    stream.write(&msg_size).unwrap();

    // Send the message.
    stream.write(&res.data).unwrap();
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

fn gen_route(user_ip: &str, key: &KeyArr) -> Vec<(String, KeyArr)> {
    vec![(user_ip.to_string(), key.clone())]
}

fn generate_route(users: &HashMap<String, KnownUser>, dest: (String, KeyArr)) -> Vec<(String, KeyArr)> {
    let mut r = vec![];
    let n = cmp::min(3, users.len());
    for v in users.values().take(n) {
        r.push((v.addr.clone(), v.public_key.clone()))
    }
    r.push(dest);
    r
}

fn login_response(username: String, password: String, users: &UserMap, usr_ip: String, crypto: &Crypto, key: &KeyArr) -> Message {
    let route = gen_route(&usr_ip, &key);
    match users.lock().unwrap().get(&username) {
        Some(u) => {
            if *password == u.password {
                Message::new(
                    MessageType::User(
                        ToUser::ServerResponse(
                            ResponseType::User ( 
                                User {
                                    handle: u.handle.clone(),
                                    addr: usr_ip,
                                    public_key: u.public_key.clone(),
                                }
                            )
                        )
                    ),
                    route,
                    &crypto
                )
            } else {
                Message::new(
                    MessageType::User(
                        ToUser::ServerResponse(
                            ResponseType::Error("Incorrect password.".to_string())
                        )
                    ),
                    route,
                    &crypto
                )
            }
        },
        None => {
            Message::new(
                MessageType::User(
                    ToUser::ServerResponse(
                        ResponseType::Error("User does not exist.".to_string())
                    )
                ),
                route,
                &crypto
            )
        }
    }
}

fn register_response(username: String, password: String, public_key: [u8; 32], users: &UserMap, usr_ip: String, crypto: &Crypto, key: &KeyArr) -> Message {
    let route = gen_route(&usr_ip, &key);
    let ref mut users = *users.lock().unwrap();
    // this can probably be simplified using users.entry()
    match users.get(&username) {
        Some(_) => Message::new(
            MessageType::User(ToUser::ServerResponse(ResponseType::Error (
                "Username already in use.".to_string()
            ))),
            route,
            &crypto
        ),
        None => {
            users.insert(username.clone(), KnownUser {
                handle: username.clone(),
                addr: usr_ip.clone(),
                password: password.clone(),
                public_key: public_key.clone()
            });
            Message::new(
                MessageType::User(
                    ToUser::ServerResponse(
                        ResponseType::User(
                            User {
                                handle: username.clone(),
                                addr: usr_ip,
                                public_key: public_key.clone()
                            }
                        )
                    )
                ), 
                route,
                &crypto
            )
        }
    }
}

fn connect_response(name: String, users: &UserMap, route: Vec<(String, KeyArr)>, crypto: &Crypto) -> Message {
    let ref users = *users.lock().unwrap();
    match users.get(&name) {
        Some(known_user) => Message::new(
            MessageType::User(
                ToUser::ServerResponse(
                    ResponseType::Connection(
                        UserInfo {
                            //route: vec![(known_user.addr.clone(), known_user.public_key.clone())],
                            route: generate_route(users, (known_user.addr.clone(), known_user.public_key.clone())),
                            addr: known_user.addr.clone(),
                            public_key: known_user.public_key.clone()
                        }
                    )
                )
            ),
            route,
            &crypto
        ),
        None => Message::new(
            MessageType::User(
                ToUser::ServerResponse(
                    ResponseType::Error(format!("Could not find user {}.", name))
                )
            ),
            route,
            &crypto
        )
    }
}

fn create_response(msg: &Message, users: &UserMap, stream: &TcpStream, crypto: &Crypto) -> Result<Message, ()> {
    let usr_ip = addr_to_string(&stream);
    if let MessageType::Server(ref msg) = Net::data_to_type(&msg.data) {
        match *msg {
            ToServer::Login(ref username, ref password, ref public_key) =>
                Ok(login_response(username.clone(), password.clone(), &users, usr_ip, &crypto, public_key)),
            ToServer::Register(ref username, ref password, ref public_key) =>
                Ok(register_response(username.clone(), password.clone(), *public_key, &users, usr_ip, &crypto, public_key)),
            ToServer::Connect(ref name, ref public_key) =>
                Ok(connect_response(name.clone(), &users, gen_route(&usr_ip, &public_key), &crypto)),
            ToServer::PublicKey(_) =>
                Err(())
        }
    } else {
        Err(())
    }

}


fn handler(mut stream: TcpStream, users: UserMap, crypto: Crypto) {
    let msg: Message = receive_message(&mut stream, &crypto);
    let response = create_response(&msg, &users, &stream, &crypto).unwrap();
    send_response(stream, response);
}

fn pub_key_handler(mut stream: TcpStream, pubkey: [u8; 32], crypto: &Crypto) {
    let usr_ip = addr_to_string(&stream);
    let msg_type: MessageType = receive_unencrypted_message_type(&mut stream);
    let response = match msg_type {
        MessageType::Server(mt) => {
            match mt {
                ToServer::PublicKey(pk) => {
                    Message::new(
                        MessageType::User(
                            ToUser::ServerResponse(
                                ResponseType::PublicKey(pubkey)
                            )
                        ),
                        gen_route(&usr_ip, &pk),
                        &crypto
                    )
                },
                _ => return
            }
        },
        _ => return
    };
    send_response(stream, response);
}

















