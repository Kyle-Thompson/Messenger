use std::io::{self, Write};

//extern crate ncurses;
//use self::ncurses::*;
use net_lib::TextMessage;

/*
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
}*/

pub struct IOHandler {
    //print_lock: Arc<(Mutex<()>)>,
    //prompt: Arc<(Mutex<Prompt>)>,
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
            //print_lock: Arc::new(Mutex::new(())),
            //prompt: Arc::new(Mutex::new(Prompt::new())),
        }
    }

    pub fn read_line(&self, mut string: &mut String) {
        io::stdin().read_line(&mut string).expect("Failed to read user input.");
    }

    pub fn read_prompted_line(&self, mut string: &mut String, prompt: &str) {
        print!("{}", prompt);
        //print!("{}", self.prompt.lock().unwrap().get_prompt());
        io::stdout().flush().expect("Could not flush buffer.");
        self.read_line(&mut string);
    }

    pub fn print_new_message(&self, msg: TextMessage) {
        println!("{}", msg);
        io::stdout().flush().expect("Could not flush buffer.");
    }
    
    pub fn print_error(&self, err: &'static str) {
        println!("{}", err);
        io::stdout().flush().expect("Could not flush buffer.");
    }
}

impl Drop for IOHandler {
    fn drop(&mut self) {
        //endwin();
    }
}

