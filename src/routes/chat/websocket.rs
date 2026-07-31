use super::*;
use crate::routes::{http_utils::{get_ws_key}, chat::models::{ChatMessage, RawChatMessage}};
use crate::routes::db::save_to_db;
use auth::sessions::{get_session_id, verify_session};
use tokio::select;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::Role;
use sha1::{Sha1, Digest};
use base64::{engine::general_purpose, Engine as _};
use futures_util::{StreamExt, SinkExt};
use tokio::sync::broadcast::Sender;
use sqlx::mysql::MySqlPool;
use serde_json::from_str;
use tokio::signal;
use crate::routes::http_utils::send_json;
use crate::routes::auth::form::AuthResponse;

pub async fn handle_websocket(mut stream: TcpStream, buffer: &[u8], sender: Sender<ChatMessage>, db_pool: MySqlPool, shutdown: Sender<ShutDown>) {
    let ws_key = get_ws_key(String::from_utf8_lossy(&buffer).to_string()).unwrap();
    let session_id = get_session_id(&buffer);
    let value = verify_session(session_id, db_pool.clone()).await;
    if value.is_none() {
        send_json(
            AuthResponse::from_err("Session is required to start a websocket connection"),
            "404",
            "Unauthorized",
            None,
            stream,
        ).await;
        return;
    }
    let (username, color) = value.unwrap();
    let combined = ws_key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let sha1_hashed = Sha1::digest(combined.as_bytes());
    let result = general_purpose::STANDARD.encode(sha1_hashed);
    let response = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n", result);
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.flush().await;
    let ws_stream = WebSocketStream::from_raw_socket(stream, Role::Server, None).await;
    let (mut write, mut read) = ws_stream.split();
    let mut rx = sender.subscribe();
    loop {
        select! {
            Some(result) = read.next() => {
                match result {
                    Ok(msg) => {
                        let str = msg.to_text().unwrap();
                        let raw: RawChatMessage = match from_str(str) {
                            Ok(v) => v,
                            Err(e) => {
                                log(&format!("failed to parse message: {}", e), "red", false);
                                continue;
                            }
                        };
                        let message = ChatMessage::from_raw(raw, &username);
                        let _ = save_to_db(message.clone(), &db_pool).await;
                        let _ = sender.send(message);
                    }
                    Err(e) => {
                        log(&format!("connection error: {}", e), "red", false);
                        break;
                    }
                }
            }
            Ok(internal_msg) = rx.recv() => {
                let serialized = serde_json::to_string(&internal_msg).unwrap();
                let msg = Message::text(serialized);
                let _ = write.send(msg).await;
            }
            Ok(_) = signal::ctrl_c() => {
                log("Shutting down...", "green", false);
                break;
            }
        };
    }
    let _ = write.send(Message::Close(None)).await;
    log("Stopped websocket loop", "green", false);
    drop(write);
    drop(read);
    log("dropped channel", "green", false);
    let _ = shutdown.send(ShutDown);
    log("Sent shutdown signal to main loop", "green", false);
}