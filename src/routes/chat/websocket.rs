use crate::http_utils::get_ws_key;
use crate::db::save_to_db;
use crate::http_utils::{send_json, finish_handshake};
use crate::routes::auth::form::AuthResponse;
use crate::log;
use crate::Shutdown;
use super::models::{ChatMessage, RawChatMessage};
use crate::session_utils::{get_session_id, verify_session};

use tokio::select;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::Role;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::broadcast::Sender;
use tokio::net::TcpStream;
use sqlx::mysql::MySqlPool;
use serde_json::from_str;
use tokio::signal;

pub async fn handle_websocket(mut stream: TcpStream, buffer: &[u8], sender: Sender<ChatMessage>, db_pool: MySqlPool, shutdown: Sender<Shutdown>) {
    let ws_key = match get_ws_key(String::from_utf8_lossy(&buffer).to_string()) {
        Some(key) => key,
        None => {
            send_json(
                AuthResponse::from_err("Cookie is required to start websocket connection"),
                "400",
                "BAD REQUEST",
                None,
                stream,
            ).await;
            return;
        }
    };
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
    let username = value.unwrap();
    finish_handshake(&mut stream, ws_key).await;
    let ws_stream = WebSocketStream::from_raw_socket(stream, Role::Server, None).await;
    let (mut write, mut read) = ws_stream.split();
    let mut rx = sender.subscribe();
    let mut shutdown_triggered = false;
    loop {
        select! {
            Some(result) = read.next() => {
                match result {
                    Ok(msg) => {
                        match msg {
                            Message::Text(text) => {
                                let raw: RawChatMessage = match from_str(text.as_str()) {
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
                            Message::Binary(_) => {
                                continue;
                            }
                            Message::Close(_) => {
                                let _ = write.send(Message::Close(None)).await;
                                break;
                            }
                            Message::Ping(payload) => {
                                let _ = write.send(Message::Pong(payload)).await;
                                continue;
                            }
                            _ => {
                                continue;
                            }
                        }
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
                shutdown_triggered = true;
                let _ = write.send(Message::Close(None)).await;
                log("Shutting down...", "green", false);
                break;
            }
        };
    }
    drop(write);
    drop(read);
    if shutdown_triggered {
        let _ = shutdown.send(Shutdown);
        log("Sent shutdown signal to main loop", "green", false);
    }
    log("Connection stopped", "green", false);
}