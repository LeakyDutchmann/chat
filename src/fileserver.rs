use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::path::{PathBuf, Path};


pub async fn serve_file(mut stream: TcpStream, path: &str, content_type: &str) -> std::io::Result<()> {
    let mut file = File::open(path).await?;
    let metadata = file.metadata().await?;
    let length = metadata.len();
    let header = format!("HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        content_type, length
    );
    stream.write_all(header.as_bytes()).await?;

    let mut buffer = [0; 8192];
    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break
        }
        stream.write_all(&buffer[..n]).await?;
    }
    stream.shutdown().await?;
    Ok(())
}

//This function is a guarantee to get correct file pathes no mater if program is running from docker or by manual setup
pub fn get_path(file: &str) -> String {
    let app_root = std::env::var("APP_ROOT").unwrap_or_else(|_| ".".to_string());
    if let Some(path) = PathBuf::from(app_root).join("static").join(file).to_str() {
        println!("path: {}", path);
        path.to_string()
    } else {
        "".to_string()
    }
    
}