use super::*;
use std::io::Write;

pub fn serve_file(mut stream: TcpStream, path: &str, content_type: &str) -> std::io::Result<()> {
    let file = std::fs::read_to_string(path)?;
    let length = file.len();
    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        content_type, length, file
    );
    stream.write(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

