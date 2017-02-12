//extern crate pancurses;

use std::io::{self, Write};
use std::sync::mpsc::Sender;
//use pancurses::*;

pub struct InputHandlerSenders {
    pub done: Sender<i32>,      // extra input done
    pub output: Sender<String>, // input handler to output
    pub sender: Sender<String>, // input handler to sender
}

pub struct IOHandler {
    //window: pancurses::Window,
}

impl IOHandler {
    pub fn new() -> IOHandler {
        println!("initializing io handler");
        io::stdout().flush().expect("Could not flush buffer.");

        IOHandler{
        //    window : pancurses::initscr(),
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

