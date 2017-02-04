use std::io::{self, Write};

pub fn read_line(mut string: &mut String) {
    io::stdout().flush().expect("Could not flush buffer.");
    io::stdin().read_line(&mut string).expect("Failed to read user input!");
}

pub fn read_prompted_line(mut string: &mut String, prompt: &str) {
    print!("{}", prompt);
    read_line(&mut string);
}

