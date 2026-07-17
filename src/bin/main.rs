use chat::routes;
use colored::Colorize;
use routes::ChatMessage;

use std::io::ErrorKind::UnexpectedEof;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast::Sender;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let (tx, _rx) = tokio::sync::broadcast::channel::<ChatMessage>(1024);
    println!("Listening on 127.0.0.1:8080");

    loop {
        let (mut stream, addr) = listener.accept().await?;
        let tx_cloned = tx.clone();
        println!("Accepted connection from {}", addr);
        
        tokio::spawn(async move {
            let mut buffer = [0u8; 4096];
            let n = match stream.read(&mut buffer).await {
                Ok(0) => {
                    println!("Connection with: {} is lost", addr);
                    return;
                }
                Ok(n) => {
                    n
                },
                Err(e) => {
                    println!("Error reading a buffer: {}", e);
                    return;
                }
            };
            routes::handle_routes(stream, &buffer[0..n], tx_cloned).await;
        });
    }
}


