use std::sync::Arc;

use clap::Parser;
use kernel::adapters::discovery;
use kernel::adapters::http::server::start_server;
use reqwest::Client;

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
        println!("Found reciever! Starting sending files!");
        let url = format!("http://{}:8080/upload", reciever.ip());
        upload_file(&url, &args.file).await.unwrap();
        //let file = tokio::fs::read(args.file).await.unwrap();
    }
}

pub async fn upload_file(url: &str, path: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let form = reqwest::multipart::Form::new()
        .file("file", path)
        .await
        .unwrap();

    client.post(url).multipart(form).send().await?;

    Ok(())
}
