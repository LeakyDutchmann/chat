use super::*;
use crate::fileserver::serve_file;
use tokio::sync::broadcast::Sender;
use urlencoding::decode;
use serde::Serialize;
use sqlx::{self, mysql::{MySqlPool, MySqlRow}, query, FromRow, Row};


#[derive(Clone, Debug)]
pub enum Route {
    Init,
    Js,
    CssReset,
    StyleCss,
    Icon,
    Events,
    History,
    Message(String),
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
    RouteEntry { path: b"GET /events HTTP/1.1", route: Route::Events},
    RouteEntry { path: b"GET /history HTTP/1.1", route: Route::History },
];

impl Route {
    pub fn from_buffer(buffer: &[u8]) -> Route {
        for route_ent in ROUTES {
            if buffer.starts_with(route_ent.path) {
                return route_ent.route.clone();
            }
        }
        if buffer.starts_with(b"POST /message HTTP/1.1") {
            let body = parse_request_body(String::from_utf8_lossy(buffer).to_string());
            return Route::Message(body);
        }
        Route::Unexpected(String::from_utf8_lossy(buffer).to_string())
    }
}

#[derive(Clone, Serialize)]
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
        Route::Events => {
            let header = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n";
            let _ = stream.write_all(header.as_bytes()).await;
            let mut rx = sender.subscribe();
            loop {
                match rx.recv().await {
                    Ok(msg) => {
                        let json = serde_json::to_string(&msg).unwrap();
                        let data = format!("data: {}\n\n", json);
                        if stream.write_all(data.as_bytes()).await.is_err() {
                            println!("Client disconected");
                            return;
                        } else {
                            println!("message sent");
                        }
                    },
                    Err(e) => {
                        println!("Broadcast channel closed: {}", e);
                        return;
                    }
                    
                }
            }
        }
        Route::Message(form) => {
            let message = ChatMessage::from_form(form).expect("Failed to parse the message");
            let _ = save_to_db(message.clone(), &db_pool).await;
            let _ = sender.send(message);
            let response = format!("HTTP/1.1 OK 200");
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.flush().await;
            
        },
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
        Route::Unexpected(value) => {
            println!("Got unexpected route: {}", value);
        }
    }
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