use std::thread;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};

//extern crate ncurses;
//use self::ncurses::*;

struct Prompt {
    prompt: String,
}

impl Prompt {
    
    pub fn new() -> Prompt {
        Prompt {
            prompt: String::from("> "),
        }
    }

    pub fn get_prompt(&self) -> &str {
        &self.prompt
    }
}

pub struct IOHandler {
    print_lock: Arc<(Mutex<()>)>,
    prompt: Arc<(Mutex<Prompt>)>,
}

impl IOHandler {
    pub fn new() -> IOHandler {
        println!("Welcome to SecMsg! Enter '/help' to get help or '/login' to get started.");
        io::stdout().flush().expect("Could not flush buffer.");

        //initscr();
        //raw();

        //printw("Welcome to SecMsg! Enter '/help' to get help or '/login' to get started.");
        //refresh();

        IOHandler { 
            print_lock: Arc::new(Mutex::new(())),
            prompt: Arc::new(Mutex::new(Prompt::new())),
        }
    }

    pub fn read_line(&self, mut string: &mut String) {
        io::stdin().read_line(&mut string).expect("Failed to read user input.");
    }

    pub fn read_prompted_line(&self, mut string: &mut String) {
        print!("{}", self.prompt.lock().unwrap().get_prompt());
        io::stdout().flush().expect("Could not flush buffer.");
        self.read_line(&mut string);
    }

    pub fn get_line(&self) -> String {
        String::from("") // TODO
    }
}

impl Drop for IOHandler {
    fn drop(&mut self) {
        //endwin();
    }
}

