use std::io;


struct User {
    handle: String,
    password: String,
    // public_key: String,
    // private_key: String,
}

fn main() {
    
    // login
    let mut username = String::new();
    let mut password = String::new();
    loop {
        print!("username: ");
        io::stdin().read_line(&mut username);
        print!("password: ");
        io::stdin().read_line(&mut password);

        println!("Your username and password is {} and {}", username, password);
        break;
    }
    
    
    println!("Hello, world!");
}
