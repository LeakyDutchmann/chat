use super::routes::auth::form::AuthResponse;

use base64::{engine::general_purpose, Engine as _};
use sha1::{Sha1, Digest};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub fn get_ws_key(request: String) -> Option<String> {
    let lines: Vec<String> = request.lines().map(|l| l.to_string()).collect();
    for line in &lines {
        if line.starts_with("Sec-WebSocket-Key: ") {
            return Some(line[19..].to_string());
        }
    }
    None
}

pub fn parse_request_body(request: String) -> String {
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

pub async fn send_json(
    status: AuthResponse,
    status_code: &str,
    status_message: &str,
    additional_header: Option<&str>,
    mut stream: TcpStream
) {
    let status_str = serde_json::to_string(&status).unwrap();
    let len = status_str.len();
    let mut response = format!("HTTP/1.1 {} {}\r\nContent-Type application/json\r\nContent-Length {}\r\n", status_code, status_message, len);

    if let Some(additional_header) = additional_header {
        response.push_str(additional_header);
        response.push_str("\r\n");
    }

    response.push_str("\r\n");
    response.push_str(&status_str);

    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.flush().await;
}

pub async fn finish_handshake(stream: &mut TcpStream, ws_key: String) {
    let combined_key = ws_key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let sha1_hashed_key = Sha1::digest(combined_key.as_bytes());
    let final_key = general_purpose::STANDARD.encode(sha1_hashed_key);
    let response = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n", final_key);
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.flush().await;
}
