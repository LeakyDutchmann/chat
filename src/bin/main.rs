use chat::routes::routes;
use colored::Colorize;
use routes::ChatMessage;

use std::io::ErrorKind::UnexpectedEof;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast::Sender;
use std::time::Duration;
use sqlx::{self, mysql::{MySqlPoolOptions, MySqlPool}};
use tokio::signal;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let (tx, _rx) = tokio::sync::broadcast::channel::<ChatMessage>(1024);
    let db_url = "mysql://root:Tima1405pereviz@localhost:3306/chat_db";
    let db_pool = estabilish_connection(db_url).await.unwrap();
    println!("Listening on 127.0.0.1:8080");
    loop {
        let pool = db_pool.clone();
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
            routes::handle_routes(stream, &buffer[0..n], tx_cloned, pool).await;
        });
    }
}

pub async fn estabilish_connection(db_url: &str) -> anyhow::Result<MySqlPool> {
    let pool = MySqlPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(10))
        .connect(db_url)
        .await?;
    Ok(pool)
}


