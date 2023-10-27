use clap::Parser;
use http_server_starter_rust::client::handle_client;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    directory: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let args = Args::parse();

    let dir = Arc::new(Mutex::new(args.directory));

    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        let dir = dir.clone();
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let _ = handle_client(socket, dir).await;
        });
    }
}
