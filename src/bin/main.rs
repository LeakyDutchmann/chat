use chat::routes::routes;
use chat::routes::chat::models::ChatMessage;
use chat::routes::db::estabilish_connection;
use chat::routes::auth::sessions::cleanup_sessions;
use chat::ShutDown;
use colored::Colorize;


use std::io::ErrorKind::UnexpectedEof;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast::Sender;
use sqlx::{self, mysql::{MySqlPoolOptions, MySqlPool}};
use tokio::signal;
use tokio::select;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

pub async fn shut_down() -> bool {
    let result = signal::ctrl_c().await;
    if result.is_ok() {
        return true;
    }
    false
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<ShutDown>(1024);
    let mut shutdown_rx = shutdown_tx.subscribe();
    let (tx, _rx) = tokio::sync::broadcast::channel::<ChatMessage>(1024);
    let db_url = "mysql://root:Tima1405pereviz@localhost:3306/chat_db";
    let db_pool = estabilish_connection(db_url).await.unwrap();
    println!("Listening on 127.0.0.1:8080");
    loop {
        let shutdown_tx = shutdown_tx.clone();
        let pool = db_pool.clone();
        select! {
            Ok(result) = listener.accept() => {
                let (mut stream, addr) = result;
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
                    routes::handle_routes(stream, &buffer[0..n], tx_cloned, pool, shutdown_tx).await;
                });
            }
            Ok(_) = shutdown_rx.recv() => {
                println!("Stopping main loop...");
                break;
            }
        };
    }
    cleanup_sessions(&db_pool).await;
    println!("Graceful shutdown completed");
    Ok(())
}