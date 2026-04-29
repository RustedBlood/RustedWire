use kernel::application::ports;
use std::io::{self, Read};

pub struct UserService {}

impl ports::UserInteractionService for UserService {
    fn ask_accept_files(&self, sender_info: &kernel::domain::transfer::SenderInfo) -> bool {
        let mut input = String::new();
        println!("Do you want to accept files (y/n)?");
        io::stdin()
            .read_to_string(&mut input)
            .expect("Failed to read(");
        input.trim();
        if &input == "y" {
            return true;
        }
        return false;
    }
}
