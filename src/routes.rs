use super::*;
use crate::fileserver::serve_file;
use tokio::sync::broadcast::Sender;
use urlencoding::decode;
use serde::{Serialize, Deserialize};
use sqlx::{self, mysql::{MySqlPool, MySqlRow}, query, FromRow, Row};
use serde_json::from_str;
use futures_util::{StreamExt, SinkExt};
use tokio::select;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::Role;
use sha1::{Sha1, Digest};
use base64::{engine::general_purpose, Engine as _};


#[derive(Clone, Debug)]
pub enum Route {
    Init,
    Js,
    CssReset,
    StyleCss,
    Icon,
    WebSocket,
    History,
    Unexpected(String)
}

struct RouteEntry {
    route: Route,
    path: &'static [u8]
}

static ROUTES: &[RouteEntry] = &[
    RouteEntry { path: b"GET / HTTP/1.1", route: Route::Init },
    RouteEntry { path: b"GET /script.js HTTP/1.1", route: Route::Js },
    RouteEntry { path: b"GET /reset.css HTTP/1.1", route: Route::CssReset },
    RouteEntry { path: b"GET /style.css HTTP/1.1", route: Route::StyleCss },
    RouteEntry { path: b"GET /favicon.ico HTTP/1.1", route: Route::Icon },
    RouteEntry { path: b"GET /history HTTP/1.1", route: Route::History },
    RouteEntry { path: b"GET /ws HTTP/1.1", route: Route::WebSocket}
];

// static ROUTES: &[RouteEntry] = &[
//     RouteEntry { path: b"GET / HTTP/1.1", route: Route::Init },
//     RouteEntry { path: b"GET /script.js HTTP/1.1", route: Route::Js },
//     RouteEntry { path: b"GET /reset.css HTTP/1.1", route: Route::CssReset },
//     RouteEntry { path: b"GET /style.css HTTP/1.1", route: Route::StyleCss },
//     RouteEntry { path: b"GET /favicon.ico HTTP/1.1", route: Route::Icon },
//     RouteEntry { path: b"GET /events HTTP/1.1", route: Route::Events},
//     RouteEntry { path: b"GET /history HTTP/1.1", route: Route::History },
// ];

impl Route {
    pub fn from_buffer(buffer: &[u8]) -> Route {
        for route_ent in ROUTES {
            if buffer.starts_with(route_ent.path) {
                return route_ent.route.clone();
            }
        }
        Route::Unexpected(String::from_utf8_lossy(buffer).to_string())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub room: String,
    pub username: String,
    pub message: String
}

impl ChatMessage {
    pub fn from_form(form: String) -> Option<ChatMessage> {
        let parts: Vec<String> = form.split("&").map(|p| p.to_string()).collect();
        let mut parsed = ChatMessage {
            room: String::new(),
            username: String::new(),
            message: String::new(),
        };
        for part in parts {
            if let Some((a, b)) = part.split_once("=") {
                let mut decoded = decode(b.trim()).ok()?.to_string();
                decoded = decoded.replace("+", " ");
                if decoded.contains('+') {
                    println!("invalid character: {}", decoded);
                }
                match a {
                    "room" => parsed.room = decoded,
                    "username" => parsed.username = decoded,
                    "message" => parsed.message = decoded,
                    _ => { continue}
                }
            }
        }
        Some(parsed)
    }
}

impl FromRow<'_, MySqlRow> for ChatMessage {
    fn from_row(row: &MySqlRow) -> Result<ChatMessage, sqlx::Error> {
        let room: String = row.try_get("room")?;
        let username: String = row.try_get("username")?;
        let message: String = row.try_get("message")?;
        Ok(ChatMessage {
            room: room,
            username: username,
            message: message,
        })
    }
}


pub async fn handle_routes(mut stream: TcpStream, buffer: &[u8], sender: Sender<ChatMessage>, db_pool: MySqlPool) {
    let route = routes::Route::from_buffer(&buffer);
    match route {
        Route::Init => {
            let result = serve_file(stream, "static/index.html", "text/html").await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::Js => {
            let result = serve_file(stream, "static/script.js", "application/javascript").await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::CssReset => {
            let result = serve_file(stream, "static/reset.css", "text/css").await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::StyleCss => {
            let result = serve_file(stream, "static/style.css", "text/css").await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::Icon => {
            return;
        }
        Route::History => {
            let rows_opt: Option<Vec<ChatMessage>> = sqlx::query_as("SELECT * FROM messages")
                .fetch_all(&db_pool)
                .await.ok();
            if let Some(rows) = rows_opt {
                let json = serde_json::to_string(&rows).unwrap();
                let response = format!("HTTP/1.1 OK 200\r\nContent-Type: application/json\r\n\r\n{}", json);
                let _ = stream.write_all(response.as_bytes()).await;
                let _ = stream.flush().await;
            }
        },
        Route::WebSocket => {
            let ws_key = get_ws_key(String::from_utf8_lossy(&buffer).to_string()).unwrap();
            let combined = ws_key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
            let sha1_hashed = Sha1::digest(combined.as_bytes());
            let result = general_purpose::STANDARD.encode(sha1_hashed);
            println!("key: {}", combined);
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
                                let message: ChatMessage = match from_str(str) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        println!("failed to parse message: {}", e);
                                        continue;
                                    }
                                };
                                let _ = save_to_db(message.clone(), &db_pool).await;
                                let _ = sender.send(message);
                            }
                            Err(e) => {
                                println!("connection error: {}", e);
                                break;
                            }
                        }
                    }
                    Ok(internal_msg) = rx.recv() => {
                        println!("Got bc msg");
                        let serialized = serde_json::to_string(&internal_msg).unwrap();
                        let msg = Message::text(serialized);
                        let _ = write.send(msg).await;
                    }
                };
            }
        }
        Route::Unexpected(value) => {
            println!("Got unexpected route: {}", value);
        }
    };
}

pub async fn save_to_db(message: ChatMessage, db: &MySqlPool) -> anyhow::Result<()> {
    let result = sqlx::query("INSERT INTO messages(room, username, message) VALUES(?, ?, ?)")
        .bind(message.room)
        .bind(message.username)
        .bind(message.message)
        .execute(db)
        .await?;
    if result.rows_affected() > 0 {
        println!("Message saved to database");
    }
    Ok(())
}

pub fn get_ws_key(request: String) -> Option<String> {
    let lines: Vec<String> = request.lines().map(|l| l.to_string()).collect();
    for line in &lines {
        if line.starts_with("Sec-WebSocket-Key: ") {
            return Some(line[19..].to_string());
        }
    }
    None
}

fn parse_request_body(request: String) -> String {
    let lines: Vec<String> = request.lines().map(|l| l.to_string()).collect();
    let mut collect = false;
    let mut body = String::from("");
    for line in &lines {
        if collect {
            body.push_str(line);
        }
        if line.is_empty() {
            collect = true;
        }
    }
    body
}
