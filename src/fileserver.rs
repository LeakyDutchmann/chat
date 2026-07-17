use super::*;
use tokio::fs::File;

pub async fn serve_file(mut stream: TcpStream, path: &str, content_type: &str) -> std::io::Result<()> {
    let mut file = File::open(path).await?;
    let metadata = file.metadata().await?;
    let length = metadata.len();
    let header = format!("HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        content_type, length
    );
    let _ = stream.write_all(header.as_bytes()).await?;

    let mut buffer = [0; 8192];
    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break
        }
        let _ = stream.write_all(&buffer[..n]).await?;
    }
    stream.shutdown().await?;
    Ok(())
}

