use chat::{routes, threadpool::ThreadPool};

use std::io::ErrorKind::UnexpectedEof;
use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::io::Write;


fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:80")?;
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream?;
        pool.execute(|| {
            handle_connection(stream);
        });
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    routes::handle_route(stream, buffer);
    
}

