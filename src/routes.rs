use super::*;
use crate::fileserver::serve_file;


#[derive(Clone)]
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
            return Route::Message(String::from_utf8_lossy(&buffer).to_string());
        }
        Route::Unexpected(String::from_utf8_lossy(buffer).to_string())
    }
}

#[derive(Clone)]
struct ChatMessage {
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
                match a {
                    "room" => parsed.room = b.to_string(),
                    "username" => parsed.username = b.to_string(),
                    "message" => parsed.message = b.to_string(),
                    _ => {return None}
                }
            }
        }
        Some(parsed)
    }
}

pub fn handle_route(mut stream: TcpStream, buffer: [u8; 1024]) {
    let route = Route::from_buffer(&buffer);
    let (tx, _rx) = tokio::sync::broadcast::channel::<ChatMessage>(1024);
    match route {
        Route::Init => {
            let result = serve_file(stream, "static/index.html", "text/html");
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::Js => {
            let result = serve_file(stream, "static/script.js", "application/javascript");
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::CssReset => {
            let result = serve_file(stream, "static/reset.css", "text/css");
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::StyleCss => {
            let result = serve_file(stream, "static/style.css", "text/css");
            match result {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to serve file: {}", e);
                }
            }
        }
        Route::Icon => {
            println!("got icon request");
        }
        Route::Events => {
            let response = format!("HTTP/1.1 OK 200\r\nContent-type: text/event-stream\r\nCashe-Control: no-cache\r\nConnection: keep-alive\r\n\r\n");
            stream.write(response.as_bytes()).ok();
            stream.flush().ok();
            let mut rx_sub = tx.subscribe();
            tokio::spawn(async move{
                loop {
                    while let Ok(message) = rx_sub.recv().await {
                        let data = format!("data: {}\n\n", message.message);
                        stream.write(data.as_bytes()).ok();
                        stream.flush().ok();
                    }
                }
                
            });
        }
        Route::Message(request) => {
            let body = parse_request_body(request);
            if let Some(message) = ChatMessage::from_form(body) {
                tx.send(message).ok();
                let response = format!("HTTP/1.1 OK 200");
                stream.write(response.as_bytes()).ok();
                stream.flush().ok();
            }
        }
        Route::Unexpected(req) => {
            println!("got unexpected request: {}", req);
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