use clap::Parser;
use kernel::adapters::discovery;
use kernel::adapters::http::server::start_server;
use reqwest::Client;
use std::io::{self, Write};
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
    if args.reciver {
        println!("Started to recieve");
        tokio::spawn(async move {
            start_server().await;
        });
        discovery::reciever::start_to_recieve().await.unwrap();
    } else if args.sender {
        println!("Starting to search recievers...");
        let reciever = discovery::sender::search_recivers().await.unwrap();
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
