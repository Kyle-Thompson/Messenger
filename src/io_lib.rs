#![allow(dead_code)]

use std::io::{self, Write};

use messages::TextMessage;

pub struct IOHandler;

impl IOHandler {
    pub fn new() -> IOHandler {
        println!("Welcome to SecMsg! Enter '/help' to get help or '/login' to get started.");
        io::stdout().flush().expect("Could not flush buffer.");

        IOHandler { }
    }

    pub fn read_line(&self, mut string: &mut String) {
        io::stdin().read_line(&mut string).expect("Failed to read user input.");
    }

    pub fn read_prompted_line(&self, prompt: &str) -> String {
        let mut string = "".to_string();
        print!("{}", prompt);
        io::stdout().flush().expect("Could not flush buffer.");
        self.read_line(&mut string);
        string.trim().to_string()
    }

    pub fn print_message(&self, msg: TextMessage) {
        println!("{}", msg.to_string());
        io::stdout().flush().expect("Could not flush buffer.");
    }

    pub fn print_messages(&self, msgs: Vec<TextMessage>) {
        for m in msgs {
            self.print_message(m);
        }
    }

    pub fn print_conversations(&self, convs: Vec<String>) {
        println!("Conversations");
        for c in convs {
            println!("{}", c);
        }
        io::stdout().flush().expect("Could not flush buffer.");
    }
    
    pub fn print_log(&self, text: &str) {
        println!("{}", text);
        io::stdout().flush().expect("Could not flush buffer.");
    }
    
    pub fn print_error(&self, err: &str) {
        println!("Error: {}", err);
        io::stdout().flush().expect("Could not flush buffer.");
    }
}

