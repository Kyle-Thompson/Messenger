mod io_lib;
mod net_lib;

static NET : &'static UdpSocket = UdpSocket::bind("127.0.0.1:5000").expect("Couldn't bind to socket");

struct User {
    handle: String,
    password: String,
    // public_key: String,
    // private_key: String,
}

fn main() {
    let user = login();
    println!("{}, {}", user.handle, user.password);

}

fn login() -> User {
    
    let mut username = String::new();
    let mut password = String::new();

    loop {
        io_lib::read_prompted_line(&mut username, "username: ");
        io_lib::read_prompted_line(&mut password, "password: ");

        break;
    }

    User { 
        handle: username.trim().to_string(), 
        password: password.trim().to_string()
    }
}
