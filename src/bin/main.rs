use chat::routes::routes;
use chat::routes::chat::models::ChatMessage;
use chat::db::estabilish_connection;
use chat::session_utils::cleanup_sessions;
use chat::Shutdown;
use chat::log;

use tokio::net::{TcpListener};
use tokio::io::{self, AsyncReadExt};
use tokio::select;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<Shutdown>(1024);
    let mut shutdown_rx = shutdown_tx.subscribe();
    let (tx, _rx) = tokio::sync::broadcast::channel::<ChatMessage>(1024);
    let db_url = "mysql://root:Tima1405pereviz@localhost:3306/chat_db";
    let db_pool = estabilish_connection(db_url).await.unwrap();
    log("Listening on 127.0.0.1:8080", "blue", true);
    loop {
        let shutdown_tx = shutdown_tx.clone();
        let pool = db_pool.clone();
        select! {
            Ok(result) = listener.accept() => {
                let (mut stream, addr) = result;
                let tx_cloned = tx.clone();
                
                let line = format!("Accepted connection from {}", addr);
                log(&line, "blue", false);
                
                tokio::spawn(async move {
                    let mut buffer = [0u8; 4096];
                    let n = match stream.read(&mut buffer).await {
                        Ok(0) => {
                            log(format!("Connection with: {} is lost", addr).as_str(), "red", false);
                            return;
                        }
                        Ok(n) => {
                            n
                        },
                        Err(e) => {
                            log(format!("Error reading a buffer: {}", e).as_str(), "red", false);
                            return;
                        }
                    };
                    routes::handle_routes(stream, &buffer[0..n], tx_cloned, pool, shutdown_tx).await;
                });
            }
            Ok(_) = shutdown_rx.recv() => {
                log("Stopping main loop...", "green", false);
                break;
            }
        };
    }
    cleanup_sessions(&db_pool).await;
    log("Graceful shutdown completed", "green", false);
    Ok(())
}