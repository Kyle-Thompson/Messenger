use std::net::{UdpSocket, Ipv4Addr, SocketAddrV4};

static SERVER_IP : &'static str = "192.168.0.1";
static SERVER_PORT : &'static str = "5000";

pub struct Net {
    socket: UdpSocket,
}

impl Net {

    pub fn new() -> Net {
        
        Net {
            socket: UdpSocket::bind("127.0.0.1:5000").expect("Couldn't bind to socket!")
        }
    }

    //fn authenticate_user(&self, username: &String, password: &String) {
        
    //}
}
