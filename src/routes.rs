use super::*;
use crate::fileserver::serve_file;
use tokio::sync::broadcast::Sender;
use urlencoding::decode;
use serde::Serialize;


#[derive(Clone, Debug)]
pub enum Route {
    Init,
    Js,
    CssReset,
    StyleCss,
    Icon,
    Events,
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

pub async fn handle_routes(mut stream: TcpStream, buffer: &[u8], sender: Sender<ChatMessage>) {
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
            let _ = sender.send(message);
            let response = format!("HTTP/1.1 OK 200");
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.flush().await;
        }
        Route::Unexpected(value) => {
            println!("Got unexpected route: {}", value);
        }
    }
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