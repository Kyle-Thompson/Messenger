use std::thread;
use std::io::{self, Write};
use std::sync::mpsc::{channel, Sender};

pub struct InputHandlerSenders {
    pub input : Sender<i32>,      // extra input done
    pub output: Sender<String>, // input handler to output
    pub sender: Sender<String>, // input handler to sender
}

pub struct IOHandler {
}

impl IOHandler {
    pub fn new() -> IOHandler {
        println!("Welcome to SecMsg! Enter '/help' to get help or '/login' to get started.");
        io::stdout().flush().expect("Could not flush buffer.");

        // output
        thread::spawn(move|| {

        });

        // input
        thread::spawn(move|| {
            let (done, recv) = channel::<i32>();

            loop {

            }
        });

        IOHandler{

        }
    }

    pub fn read_line(&self, mut string: &mut String) {
        io::stdin().read_line(&mut string).expect("Failed to read user input!");
    }

    pub fn read_prompted_line(&self, mut string: &mut String, prompt: &str) {
        print!("{}", prompt);
        io::stdout().flush().expect("Could not flush buffer.");
        self.read_line(&mut string);
    }
}

