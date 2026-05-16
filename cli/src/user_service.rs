use kernel::application::ports;
use std::io::{self, Read};

pub struct UserService {}

impl ports::UserInteractionService for UserService {
    fn ask_accept_files(&self, sender_info: &kernel::domain::transfer::SenderInfo) -> bool {
        println!(
            "Someone want to send you files!\nName: {}\nIp: {}\nFiles {:?}",
            sender_info.name, sender_info.ip, sender_info.files
        );
        println!("Do you want to accept files (y/n)?");
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .expect("Failed to read(");
        let input = input.trim();
        if input == "y" {
            return true;
        }
        return false;
    }
}
