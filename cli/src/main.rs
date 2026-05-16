use clap::Parser;
use kernel::adapters::discovery;
use kernel::adapters::discovery::sender::HostInfo;
use kernel::adapters::http::server::start_server;
use kernel::domain::transfer::{FileInfo, SenderInfo};
use reqwest::Client;
use tokio::fs::File;
use std::io::{self, Read};
use std::sync::Arc;

mod user_service;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    reciver: bool,
    #[arg(short, long)]
    sender: bool,
    #[arg(short, long)]
    file: String,
}

#[tokio::main]
async fn main() {
    eprintln!("DEBUG: Program started");
    let args = Args::parse();
    println!("{}, {}, {}", args.reciver, args.sender, args.file);
    let user_interact = Arc::new(user_service::UserService {});
    if args.reciver {
        println!("Started to recieve");
        tokio::spawn(async move {
            start_server(user_interact).await;
        });
        discovery::reciever::broadcast_send_msg().await.unwrap();
    } else if args.sender {

        println!("Starting to search recievers...");
        let reciever = discovery::sender::broadcast_get_recievers().await.unwrap();
        let recievers_len = &reciever.len();
        println!("----Choose files receivers----");
        for (i, v) in reciever.iter().enumerate() {
            let info = format!("------\n{}:\nip: {}\nname: {}\n", i, v.ip, v.name);
            println!("{}", info);
        }
        let selected_index = loop {
            println!("Please choose number from 0 to {}: ", recievers_len);
            let mut input = String::new();
            io::stdin()
                .read_to_string(&mut input)
                .expect("Failed to read(");
            let input = input.trim();
            match input.parse::<usize>() {
                Ok(num) => {
                    if &num < recievers_len {
                        break num;
                    } else {
                        println!("Not correct input!")
                    }
                }
                Err(e) => println!("Error occured: {}", e),
            }
        };
        let selected_reciever = reciever[selected_index].clone();
        prepare_files(selected_reciever);
    }
}

pub async fn prepare_files(reciever: HostInfo, files: Vec<File>) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let sneder = SenderInfo {

    }

    client.post(url).multipart(form).send().await?;

    Ok(())
}
